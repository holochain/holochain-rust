use holochain_core_types::{
    cas::content::{AddressableContent, Content},
    eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value},
    error::{HcResult, HolochainError},
    hash::HashString,
};
use std::{
    collections::{HashMap,HashSet},
    fs::{create_dir_all, File, OpenOptions},
    io::prelude::*,
    path::MAIN_SEPARATOR,
    sync::{Arc, RwLock},
};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

const ENTITY_DIR: &str = "e";
const ATTRIBUTE_DIR: &str = "a";
const VALUE_DIR: &str = "v";

#[derive(Clone, Debug)]
pub struct EavFileStorage {
    dir_path: String,
    id: Uuid,
    current_hash : HashString,
    lock: Arc<RwLock<()>>,
}

impl PartialEq for EavFileStorage {
    fn eq(&self, other: &EavFileStorage) -> bool {
        self.id == other.id
    }
}

#[warn(unused_must_use)]
pub fn add_eav_to_hashset(dir_entry: DirEntry, hash:HashString,set: &mut HashSet<HcResult<String>>) {
    let path = dir_entry.path();
    let hash = read_from_hash_list(hash).expect("Could not read index file");
    println!("read from hash {:?}",hash);
    if(hash.find(dir_entry))
    {
        match OpenOptions::new().read(true).open(path) {
        Ok(mut file) => {
            let mut content: String = String::new();
            let _result = file
                .read_to_string(&mut content)
                .map(|e| {
                    if e > 0 {
                        Ok(content)
                    } else {
                        Err(HolochainError::IoError(format!(
                            "Could not read from path {:?}",
                            path
                        )))
                    }
                })
                .map(|e| {
                    set.insert(hash,e);
                });
        }
        Err(_) => {
            set.insert(Err(HolochainError::IoError(format!(
                "Could not read from path {:?}",
                path
            ))));
        }
    }
    }
    else
    {
        ();
    }
    
}

 fn read_from_hash_list(dir_path:String,current_hash:HashString)-> Result<Vec<HashString>,HolochainError>
    {
        let path =
            vec![dir_path.clone(), current_hash].join(&MAIN_SEPARATOR.to_string());
        let file = OpenOptions::new().read(true).open(path)?;
        let file_contents = file.read_to_string()?;
        Ok(file_contents.split("\n")
                        .map(|s|{
                            HashString::from(s)
                        }).collect::<Vec<HashString>>())
    }

impl EavFileStorage {
    pub fn new(dir_path: String) -> HcResult<EavFileStorage> {
        Ok(EavFileStorage {
            dir_path,
            id: Uuid::new_v4(),
            lock: Arc::new(RwLock::new(())),
        })
    }

    fn write_to_file(
        &self,
        subscript: String,
        eav: &EntityAttributeValue,
    ) -> Result<(), HolochainError> {
        let address: String = match &*subscript {
            ENTITY_DIR => eav.entity().to_string(),
            ATTRIBUTE_DIR => eav.attribute(),
            VALUE_DIR => eav.value().to_string(),
            _ => String::new(),
        };
        let path =
            vec![self.dir_path.clone(), subscript, address].join(&MAIN_SEPARATOR.to_string());
        create_dir_all(path.clone())?;
        let address_path = vec![path, eav.address().to_string()].join(&MAIN_SEPARATOR.to_string());
        let mut f = File::create(address_path)?;
        writeln!(f, "{}", eav.content())?;
        Ok(())
    }

    fn write_to_hash_file(&self,eav:EntityAttributeValue) -> Result<(),HolochainError>
    {
        let path =
            vec![self.dir_path.clone(), self.current_hash].join(&MAIN_SEPARATOR.to_string());
        let file = OpenOptions::new()
                   .write(true)
                   .create(true)
                   .append(true)
                   .open(path);
        writeln!(file,"{}",eav.address())?;
        Ok(())
    }

   

    fn read_from_dir<T>(
        &self,
        hash:HashString,
        subscript: String,
        eav_constraint: Option<T>,
    ) -> HashMap<HashString,HcResult<String>>
    where
        T: ToString,
    {
        let address = eav_constraint
            .map(|e| e.to_string())
            .unwrap_or(String::new());
        let full_path =
            vec![self.dir_path.clone(), subscript, address].join(&MAIN_SEPARATOR.to_string());
        let mut set = HashSet::new();
        WalkDir::new(full_path.clone())
            .into_iter()
            .for_each(|dir_entry| match dir_entry {
                Ok(eav_content) => {
                    add_eav_to_hashset(eav_content, &mut set);
                }
                Err(_) => {
                    set.insert(hash,Err(HolochainError::IoError(format!(
                        "Could not obtain directory{:?}",
                        full_path
                    ))));
                }
            });

        set
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        let _guard = self.lock.write()?;
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(ENTITY_DIR.to_string(), eav)
            .and_then(|_| self.write_to_file(ATTRIBUTE_DIR.to_string(), eav))
            .and_then(|_| self.write_to_file(VALUE_DIR.to_string(), eav))
            .and_then(|_| self.write_to_hash_file(eav))
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<HashSring,EntityAttributeValue>, HolochainError> {
        let _guard = self.lock.read()?;
        let entity_set = self.read_from_dir::<Entity>(ENTITY_DIR.to_string(), entity);
        let attribute_set = self
            .read_from_dir::<Attribute>(ATTRIBUTE_DIR.to_string(), attribute)
            .clone();
        let value_set = self.read_from_dir::<Value>(VALUE_DIR.to_string(), value);

        let attribute_value_inter = attribute_set.intersection(&value_set).cloned().collect();
        let entity_attribute_value_inter: HashSet<Result<String, HolochainError>> = entity_set
            .intersection(&attribute_value_inter)
            .cloned()
            .collect();

        let maybe_first_error = entity_attribute_value_inter.iter().find(|e| e.is_err());
        if let Some(Err(first_error)) = maybe_first_error {
            return Err(first_error.to_owned());
        } else {
            let hopefully_eavs = entity_attribute_value_inter
                .iter()
                .cloned()
                .map(|maybe_eav_content|
                    // errors filtered out above... unwrap is safe.
                    Content::from(maybe_eav_content.unwrap()))
                .map(|content| EntityAttributeValue::try_from_content(&content))
                .collect::<HashSet<HcResult<EntityAttributeValue>>>();

            let maybe_first_error = hopefully_eavs.iter().find(|e| e.is_err());
            if let Some(Err(first_error)) = maybe_first_error {
                return Err(first_error.to_owned());
            } else {
                Ok(hopefully_eavs
                    .iter()
                    .cloned()
                    .map(|eav|
                        // errors filtered out above... unwrap is safe
                        eav.unwrap())
                    .collect())
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use eav::file::EavFileStorage;
    use holochain_core_types::{
        cas::{
            content::{AddressableContent, ExampleAddressableContent},
            storage::EavTestSuite,
        },
        json::RawString,
    };

    #[test]
    fn file_eav_round_trip() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let entity_content =
            ExampleAddressableContent::try_from_content(&RawString::from("foo").into()).unwrap();
        let attribute = "favourite-color".to_string();
        let value_content =
            ExampleAddressableContent::try_from_content(&RawString::from("blue").into()).unwrap();
        EavTestSuite::test_round_trip(
            EavFileStorage::new(temp_path).unwrap(),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    fn file_eav_one_to_many() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavFileStorage>(eav_storage)
    }

    #[test]
    fn file_eav_many_to_one() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavFileStorage::new(temp_path).unwrap();
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavFileStorage>(eav_storage)
    }

}
