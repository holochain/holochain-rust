pub mod actor;
use actor::{AskSelf, Protocol};
use eav::file::actor::EavFileStorageActor;
use holochain_core_types::{
    eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value},
    error::{HcResult, HolochainError},
};
use riker::actors::*;
use std::collections::HashSet;

#[derive(Clone)]
pub struct EavFileStorage {
    actor: ActorRef<Protocol>,
}

impl EavFileStorage {
    pub fn new(dir_path: String) -> HcResult<EavFileStorage> {
        Ok(EavFileStorage {
            actor: EavFileStorageActor::new_ref(&dir_path)?,
        })
    }
}

impl EntityAttributeValueStorage for EavFileStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        let response = self.actor.block_on_ask(Protocol::EavAdd(eav.clone()))?;
        unwrap_to!(response => Protocol::EavAddResult).clone()
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::EavFetch(entity, attribute, value))?;
        unwrap_to!(response => Protocol::EavFetchResult).clone()
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    use self::tempfile::tempdir;
    use eav::file::EavFileStorage;
    use holochain_core_types::cas::{
        content::{AddressableContent, ExampleAddressableContent},
        storage::EavTestSuite,
    };

    #[test]
    fn file_eav_round_trip() {
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
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
