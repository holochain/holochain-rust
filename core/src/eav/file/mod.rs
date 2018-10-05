use cas::content::AddressableContent;
use eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value};
use error::HolochainError;
use hash::HashString;
use std::{
    collections::HashSet,
    fs::{create_dir_all, read_to_string, write, File,OpenOptions},
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
        let path = vec![self.dir_path.clone(),subscript].join(&MAIN_SEPARATOR.to_string());
        create_dir_all(path.clone())?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(vec![path,self.addressable_content.address().to_string()].join(&MAIN_SEPARATOR.to_string()))?;
        writeln!(f,"{}",file_contents_to_write)?;
        Ok(())
    }

    fn read_from_file(&self, subscript: String) -> Result<String, HolochainError> {
        let filename = vec![
            self.dir_path.clone(),
            subscript,
            self.addressable_content.address().to_string(),
        ].join(&MAIN_SEPARATOR.to_string());
        let mut content = String::new();
        File::open(filename).and_then(|mut file| file.read_to_string(&mut content))?;
        Ok(content)
    }
}

impl<T> EntityAttributeValueStorage for EavFileStorage<T>
where
    T: AddressableContent,
{
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        create_dir_all(self.dir_path.clone())?;
        self.write_to_file(String::from("e".to_string()), eav.entity().to_string())
        .and_then(|_|self.write_to_file(String::from("a".to_string()), eav.attribute()))
        .and_then(|_|self.write_to_file(String::from("v".to_string()), eav.value().to_string()))
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> 
    {
        let entitySet = self.read_from_file("e".to_string()).unwrap_or_else(|_|String::new());
        let attributeSet = self.read_from_file("a".to_string()).unwrap_or_else(|_|String::new());
        let valueSet = self.read_from_file("v".to_string()).unwrap_or_else(|_|String::new());
        let eavs = entitySet
            .lines()
            .zip(attributeSet.lines().zip(valueSet.lines()))
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

#[cfg(test)]
pub mod tests {
    use cas::{content::{tests::ExampleAddressableContent, AddressableContent},memory::MemoryStorage};
    use eav::{file::EavFileStorage, EntityAttributeValue, EntityAttributeValueStorage};
    use std::collections::HashSet;
    use error::HolochainError;
    use std::fs;
    use std::{fs::{create_dir_all, read_to_string, write, File,OpenOptions},path::{Path, MAIN_SEPARATOR}};

    fn deleteFolders(path:String) -> Result<(),HolochainError>
    {
       if Path::new(&path).exists()
       {
           fs::remove_dir_all(path)?;
           Ok(())
       }
       else 
       {
            Ok(())
       }
    }

    #[test]
    fn file_eav_round_trip() 
    {
        deleteFolders(String::from("holo_round_trip")).expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_round_trip")).expect("create holo directory");
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
        let eav = EntityAttributeValue::new(
            &entity_content.address(),
            &attribute,
            &value_content.address(),
        );
        let memory_content = String::from("try this");
        let mut eav_storage = EavFileStorage::new("holo_round_trip".to_string(),memory_content).expect("should find holo file");;

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
            assert_eq!(
                expected,
                eav_storage.fetch_eav(e, a, v).expect("could not fetch eav"),
            );
        }
    }

    #[test]
    fn file_eav_one_to_many() 
    {
        deleteFolders(String::from("holo_one_to_many")).expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_one_to_many")).expect("create holo directory");
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "one_to_many".to_string();

        let memory_content = String::from("try this");
        let mut eav_storage = EavFileStorage::new("holo_one_to_many".to_string(),memory_content).unwrap();
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
        deleteFolders(String::from("holo_many_to_one")).expect("was supposed to clean up folder before test");
        create_dir_all(String::from("holo_many_to_one")).expect("create holo directory");
        let one = ExampleAddressableContent::from_content(&"foo".to_string());
        // it can reference itself, why not?
        let many_one = ExampleAddressableContent::from_content(&"foo".to_string());
        let many_two = ExampleAddressableContent::from_content(&"bar".to_string());
        let many_three = ExampleAddressableContent::from_content(&"baz".to_string());
        let attribute = "many_to_one".to_string();

        let memory_content = String::from("try this");
        let mut eav_storage = EavFileStorage::new("holo_many_to_one".to_string(),memory_content).unwrap();
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
            assert_eq!(
                expected_one,
                eav_storage
                    .fetch_eav(Some(many.address()), Some(attribute.clone()), None)
                    .expect("could not fetch eav"),
            );
        }
    }

}
