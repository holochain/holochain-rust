pub mod actor;
use crate::{
    actor::{AskSelf, Protocol},
    eav::memory::actor::EavMemoryStorageActor,
};
use holochain_core_types::{
    eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value},
    error::HolochainError,
};
use riker::actors::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct EavMemoryStorage {
    actor: ActorRef<Protocol>,
}

impl EavMemoryStorage {
    pub fn new() -> Result<EavMemoryStorage, HolochainError> {
        Ok(EavMemoryStorage {
            actor: EavMemoryStorageActor::new_ref()?,
        })
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
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
    use crate::eav::memory::EavMemoryStorage;
    use holochain_core_types::{
        cas::{
            content::{AddressableContent, ExampleAddressableContent},
            storage::EavTestSuite,
        },
        json::RawString,
    };

    #[test]
    fn memory_eav_round_trip() {
        let entity_content =
            ExampleAddressableContent::try_from_content(&RawString::from("foo").into()).unwrap();
        let attribute = "favourite-color".to_string();
        let value_content =
            ExampleAddressableContent::try_from_content(&RawString::from("blue").into()).unwrap();
        EavTestSuite::test_round_trip(
            EavMemoryStorage::new().expect("could not construct new eav memory storage"),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    fn memory_eav_one_to_many() {
        let eav_storage =
            EavMemoryStorage::new().expect("could not construct new eav memory storage");
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

    #[test]
    fn memory_eav_many_to_one() {
        let eav_storage =
            EavMemoryStorage::new().expect("could not construct new eav memory storage");
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

}
