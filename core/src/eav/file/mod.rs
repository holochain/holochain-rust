use cas::content::AddressableContent;
use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
use error::HolochainError;
use hash::HashString;
use std::{
    collections::HashSet,
    fs::{create_dir_all, read_to_string, write, File},
    io::prelude::*,
    path::{Path, MAIN_SEPARATOR},
};

pub struct EavFileStorage<T>
where
    T: AddressableContent,
{
    addressable_content: T,
    dir_path: String,
}

impl<T> EavFileStorage<T>
where
    T: AddressableContent,
{
    pub fn new(
        dir_path: String,
        addressable_content: T,
    ) -> Result<EavFileStorage<T>, HolochainError> {
        let canonical = Path::new(&dir_path).canonicalize()?;
        if !canonical.is_dir() {
            return Err(HolochainError::IoError(
                "path is not a directory or permissions don't allow access".to_string(),
            ));
        }
        Ok(EavFileStorage {
            dir_path: canonical
                .to_str()
                .ok_or_else(|| {
                    HolochainError::IoError("could not convert path to string".to_string())
                })?
                .to_string(),
            addressable_content: addressable_content,
        })
    }

    fn write_to_file(
        &self,
        subscript: String,
        file_contents_to_write: String,
    ) -> Result<(), HolochainError> {
        let path = vec![self.dir_path.clone(), String::from("e")].join(&MAIN_SEPARATOR.to_string());
        create_dir_all(path.clone())?;
        let mut f = File::create(
            vec![path.clone(), self.addressable_content.address().to_string()].join(""),
        )?;
        Ok(f.write_all(file_contents_to_write.as_bytes())?)
    }

    fn read_from_file(&self, subscript: String) -> Result<String, HolochainError> {
        let filename = vec![
            self.dir_path.clone(),
            subscript,
            self.addressable_content.address().to_string(),
        ].join(&MAIN_SEPARATOR.to_string());
        let mut content = String::new();
        let file = File::open(filename).and_then(|mut file| file.read_to_string(&mut content))?;
        Ok(content)
    }
}

impl<T> EntityAttributeValueStorage for EavFileStorage<T>
where
    T: AddressableContent,
{
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(String::from("e".to_string()), eav.entity().to_string())?;
        self.write_to_file(String::from("a".to_string()), eav.attribute())?;
        self.write_to_file(String::from("v".to_string()), eav.value().to_string())?;
        Ok(())
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        let entitySet = self.read_from_file("e".to_string())?;
        let attributeSet = self.read_from_file("a".to_string())?;
        let valueSet = self.read_from_file("v".to_string())?;
        let eavs = entitySet
            .split("\n")
            .zip(attributeSet.split("\n").zip(valueSet.split("\n")))
            .map(|f| {
                EntityAttributeValue::new(
                    &HashString::from(f.0.to_string()),
                    &(f.1).0.to_string(),
                    &HashString::from((f.1).1.to_string()),
                )
            })
            .collect::<HashSet<EntityAttributeValue>>();
        Ok(eavs
            .into_iter()
            .filter(|e| EntityAttributeValue::filter_on_eav::<Entity>(e.entity(), &entity))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Attribute>(e.attribute(), &attribute))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Value>(e.value(), &value))
            .collect::<HashSet<EntityAttributeValue>>())
    }
}
