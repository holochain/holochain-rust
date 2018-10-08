use cas::content::AddressableContent;
use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
use error::HolochainError;
use hash::HashString;
use std::{
    collections::HashSet,
    fs::{create_dir_all, read_dir, read_to_string, write, File, OpenOptions},
    io::prelude::*,
    path::{Path, MAIN_SEPARATOR},
};

use walkdir::{DirEntry, WalkDir};

pub struct EavFileStorage {
    dir_path: String,
}

impl EavFileStorage {
    pub fn new(dir_path: String) -> Result<EavFileStorage, HolochainError> {
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
        })
    }

    fn write_to_file(
        &self,
        subscript: String,
        eav: &EntityAttributeValue,
    ) -> Result<(), HolochainError> {
        let address: String = match &*subscript {
            "e" => eav.entity().to_string(),
            "a" => eav.attribute(),
            "v" => eav.value().to_string(),
            _ => String::new(),
        };
        let path =
            vec![self.dir_path.clone(), subscript, address].join(&MAIN_SEPARATOR.to_string());
        create_dir_all(path.clone())?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(vec![path, eav.address().to_string()].join(&MAIN_SEPARATOR.to_string()))?;
        writeln!(f, "{}", eav.content())?;
        Ok(())
    }

    fn read_from_dir<T>(
        &self,
        subscript: String,
        obj: Option<T>,
    ) -> HashSet<Result<String, HolochainError>>
    where
        T: ToString,
    {
        let eav_directory = match obj {
            Some(a) => a.to_string(),
            None => String::new(),
        };
        let full_path =
            vec![self.dir_path.clone(), subscript, eav_directory].join(&MAIN_SEPARATOR.to_string());
        let mut set = HashSet::new();
        WalkDir::new(full_path)
            .into_iter()
            .map(|dir_entry| match dir_entry {
                Ok(entry) => {
                    add_eav_to_hashset(entry, &mut set);
                }
                Err(_) => {
                    set.insert(Err(HolochainError::ErrorGeneric(
                        "Could not read from file".to_string(),
                    )));
                }
            })
            .collect::<HashSet<_>>();

        set
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(String::from("e".to_string()), eav)
            .and_then(|_| self.write_to_file(String::from("a".to_string()), eav))
            .and_then(|_| self.write_to_file(String::from("v".to_string()), eav))
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        let entity_set = self.read_from_dir::<Entity>("e".to_string(), entity);
        let attribute_set = self
            .read_from_dir::<Attribute>("a".to_string(), attribute)
            .clone();
        let value_set = self.read_from_dir::<Value>("v".to_string(), value);
        let attribute_value_inter = attribute_set.intersection(&value_set).cloned().collect();
        let entity_attribute_value_inter: HashSet<Result<String, HolochainError>> = entity_set
            .intersection(&attribute_value_inter)
            .cloned()
            .collect();
        Ok(entity_attribute_value_inter
            .into_iter()
            .filter(|e| e.is_ok())
            .map(|e| EntityAttributeValue::from_content(&e.unwrap()))
            .collect())
    }
}

fn add_eav_to_hashset(entry: DirEntry, set: &mut HashSet<Result<String, HolochainError>>) 
{
    OpenOptions::new()
        .read(true)
        .open(entry.path())
        .and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content).and_then(|f| {
                content
                    .lines()
                    .map(|e| set.insert(Ok(e.to_string())))
                    .collect::<HashSet<_>>();
                Ok(())
            })
        });
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::{tests::ExampleAddressableContent, AddressableContent},
        memory::MemoryStorage,
    };
    use eav::{file::EavFileStorage, EntityAttributeValue, EntityAttributeValueStorage};
    use error::HolochainError;
    use std::{
        collections::HashSet,
        fs::{self, create_dir_all, read_to_string, write, File, OpenOptions},
        path::{Path, MAIN_SEPARATOR},
    };
    use tempfile::{tempdir, TempDir};

    fn delete_folders(path: String) -> Result<(), HolochainError> {
        if Path::new(&path).exists() {
            fs::remove_dir_all(path)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    #[test]
    fn file_eav_round_trip() {
        let test_folder = "holo_round_trip";
        delete_folders(String::from("holo_round_trip"))
            .expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_round_trip")).expect("create holo directory");
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
        let eav = EntityAttributeValue::new(
            &entity_content.address(),
            &attribute,
            &value_content.address(),
        );
        let mut eav_storage =
            EavFileStorage::new("holo_round_trip".to_string()).expect("should find holo file");;

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
        delete_folders(String::from("holo_one_to_many"))
            .expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_one_to_many")).expect("create holo directory");
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "one_to_many".to_string();
        let mut eav_storage = EavFileStorage::new("holo_one_to_many".to_string()).unwrap();
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
        delete_folders(String::from("holo_many_to_one"))
            .expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_many_to_one")).expect("create holo directory");
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "many_to_one".to_string();

        let mut eav_storage = EavFileStorage::new("holo_many_to_one".to_string()).unwrap();
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
