use holochain_core_types::{
    eav::{EaviQuery, EntityAttributeValueIndex, EntityAttributeValueStorage},
    error::HolochainError,
};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::{
    collections::BTreeSet,
    fmt::{Debug, Error, Formatter},
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};
use uuid::Uuid;

const PERSISTENCE_INTERVAL: Duration = Duration::from_millis(5000);
const ENTITY : &str = "ENTITY";
const ATTRIBUTE : &str = "ATTRIBUTE";
const VALUE : &str = "VALUE";

#[derive(Clone)]
pub struct EavPickleStorage {
    db: Arc<RwLock<PickleDb>>,
    id: Uuid,
}

impl EavPickleStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> EavPickleStorage {
        EavPickleStorage {
            id: Uuid::new_v4(),
            db: Arc::new(RwLock::new(PickleDb::new(
                db_path,
                PickleDbDumpPolicy::PeriodicDump(PERSISTENCE_INTERVAL),
                SerializationMethod::Cbor,
            ))),
        }
    }
}

impl Debug for EavPickleStorage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("EavPickleStorage")
            .field("id", &self.id)
            .finish()
    }
}




impl EntityAttributeValueStorage for EavPickleStorage {
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let index_str = eav.index().to_string();
        let value = self.db.read()?.get::<EntityAttributeValueIndex>(&index_str);
        if let Some(new_eav) = value
        {
            self.add_eavi(&new_eav)
            
        }
        else 
        {
            self.db.write()?.set(&*index_str,&eav)
             .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;
            Ok(Some(eav.clone()))
        }
    
    }

    fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let inner = self.db.read()?;

        //this not too bad because it is lazy evaluated
        let entries = inner
        .iter()
        .map(|item|{
            item.get_value()      
        })
        .filter(|filter|filter.is_some())
        .map(|y|y.unwrap())
        .collect::<BTreeSet<EntityAttributeValueIndex>>();

        let entries_iter = entries.iter().cloned();
        Ok(query.run(entries_iter))
    }
}


#[cfg(test)]
pub mod tests {
    use crate::eav::pickle::EavPickleStorage;
    use holochain_core_types::{
        cas::{
            content::{AddressableContent, ExampleAddressableContent},
            storage::EavTestSuite,
        },
        eav::Attribute,
        json::RawString,
    };
    use tempfile::tempdir;

    #[test]
    fn pickle_eav_round_trip() {
        let temp = tempdir().expect("test was supposed to create temp dir");

        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let entity_content =
            ExampleAddressableContent::try_from_content(&RawString::from("foo").into()).unwrap();
        let attribute = "favourite-color".to_string();
        let value_content =
            ExampleAddressableContent::try_from_content(&RawString::from("blue").into()).unwrap();

        EavTestSuite::test_round_trip(
            EavPickleStorage::new(temp_path),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    fn pickle_eav_one_to_many() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavPickleStorage>(eav_storage);
    }

    #[test]
    fn pickle_eav_many_to_one() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavPickleStorage>(eav_storage);
    }

    #[test]
    fn pickle_eav_range() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_range::<ExampleAddressableContent, EavPickleStorage>(eav_storage);
    }

    #[test]
    fn pickle_eav_prefixes() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let eav_storage = EavPickleStorage::new(temp_path);
        EavTestSuite::test_multiple_attributes::<ExampleAddressableContent, EavPickleStorage>(
            eav_storage,
            vec!["a_", "b_", "c_", "d_"]
                .into_iter()
                .map(|p| Attribute::LinkTag(p.to_string() + "one_to_many"))
                .collect(),
        );
    }

}
