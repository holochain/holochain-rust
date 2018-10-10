use holochain_core_types::{
    cas::content::AddressableContent,
    eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value},
    error::HolochainError,
    file_validation,
};
use std::{
    collections::HashSet,
    fs::{create_dir_all, File, OpenOptions},
    io::prelude::*,
    path::MAIN_SEPARATOR,
};

use walkdir::{DirEntry, WalkDir};

type HcResult<T> = Result<T, HolochainError>;

pub struct EavFileStorage {
    dir_path: String,
}

const ENTITY_DIR: &str = "e";
const ATTRIBUTE_DIR: &str = "a";
const VALUE_DIR: &str = "v";

impl EavFileStorage {
    pub fn new(dir_path: String) -> HcResult<EavFileStorage> {
        let dir_path = file_validation::validate_canonical_path(&*dir_path)?;
        Ok(EavFileStorage { dir_path: dir_path })
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

    fn read_from_dir<T>(
        &self,
        subscript: String,
        eav_constraint: Option<T>,
    ) -> HashSet<HcResult<String>>
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
                    set.insert(Err(HolochainError::IoError(format!(
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
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(ENTITY_DIR.to_string(), eav)
            .and_then(|_| self.write_to_file(ATTRIBUTE_DIR.to_string(), eav))
            .and_then(|_| self.write_to_file(VALUE_DIR.to_string(), eav))
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
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
        Ok(entity_attribute_value_inter
            .into_iter()
            .filter(|e| e.is_ok())
            .map(|eav_content| EntityAttributeValue::from_content(&eav_content.unwrap()))
            .collect())
    }
}

fn add_eav_to_hashset(dir_entry: DirEntry, set: &mut HashSet<HcResult<String>>) {
    let path = dir_entry.path();
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
                    set.insert(e);
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

#[cfg(test)]
pub mod tests {

    use eav::file::EavFileStorage;
    use holochain_core_types::{
        cas::content::{AddressableContent, ExampleAddressableContent},
        eav::{EntityAttributeValue, EntityAttributeValueStorage},
    };
    use std::collections::HashSet;
    use tempfile::tempdir;

    #[test]
    fn file_eav_round_trip() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
        let eav = EntityAttributeValue::new(
            &entity_content.address(),
            &attribute,
            &value_content.address(),
        );
        let mut eav_storage = EavFileStorage::new(temp_path).expect("should find holo file");;

        assert_eq!(
            HashSet::new(),
            eav_storage
                .fetch_eav(
                    Some(entity_content.address()),
                    Some(attribute.clone()),
                    Some(value_content.address())
                )
                .expect("could not fetch eav"),
        );

        eav_storage.add_eav(&eav).expect("could not add eav");

        let mut expected = HashSet::new();
        expected.insert(eav.clone());
        // some examples of constraints that should all return the eav
        for (e, a, v) in vec![
            // constrain all
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                Some(value_content.address()),
            ),
            // open entity
            (None, Some(attribute.clone()), Some(value_content.address())),
            // open attribute
            (
                Some(entity_content.address()),
                None,
                Some(value_content.address()),
            ),
            // open value
            (
                Some(entity_content.address()),
                Some(attribute.clone()),
                None,
            ),
            // open
            (None, None, None),
        ] {
            println!("fetch");
            assert_eq!(
                expected,
                eav_storage.fetch_eav(e, a, v).expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn file_eav_one_to_many() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "one_to_many".to_string();
        let mut eav_storage = EavFileStorage::new(temp_path).unwrap();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&one.address(), &attribute, &many.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referencing many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &two.address(),
                    &attribute,
                    &many.address(),
                ))
                .expect("could not add eav");
        }

        // show the many results for one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(Some(one.address()), Some(attribute.clone()), None)
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &one.address(),
                &attribute.clone(),
                &many.address(),
            ));
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(None, Some(attribute.clone()), Some(many.address()))
                    .expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn file_eav_many_to_one() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "many_to_one".to_string();

        let mut eav_storage = EavFileStorage::new(temp_path).unwrap();
        let mut expected = HashSet::new();
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let eav = EntityAttributeValue::new(&many.address(), &attribute, &one.address());
            eav_storage.add_eav(&eav).expect("could not add eav");
            expected.insert(eav);
        }

        // throw an extra thing referenced by many to show fetch ignores it
        let two = ExampleAddressableContent::from_content(&"foo".to_string());
        for many in vec![many_one.clone(), many_three.clone()] {
            eav_storage
                .add_eav(&EntityAttributeValue::new(
                    &many.address(),
                    &attribute,
                    &two.address(),
                ))
                .expect("could not add eav");
        }

        // show the many referencing one
        assert_eq!(
            expected,
            eav_storage
                .fetch_eav(None, Some(attribute.clone()), Some(one.address()))
                .expect("could not fetch eav"),
        );

        // show one for the many results
        for many in vec![many_one.clone(), many_two.clone(), many_three.clone()] {
            let mut expected_one = HashSet::new();
            expected_one.insert(EntityAttributeValue::new(
                &many.address(),
                &attribute.clone(),
                &one.address(),
            ));
            println!("fetch");
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(Some(many.address()), Some(attribute.clone()), None)
                    .expect("could not fetch eav"),
            );
        }
    }

}
