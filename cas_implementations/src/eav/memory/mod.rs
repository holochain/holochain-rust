use holochain_core_types::{
    eav::{Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage, Value},
    error::HolochainError,
};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct EavMemoryStorage {
    eavs: HashSet<EntityAttributeValue>,
}

impl EavMemoryStorage {
    pub fn new() -> EavMemoryStorage {
        EavMemoryStorage {
            eavs: HashSet::new(),
        }
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        self.eavs.insert(eav.clone());
        Ok(())
    }
    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        Ok(self
            .eavs
            .iter()
            .cloned()
            .filter(|e| EntityAttributeValue::filter_on_eav::<Entity>(e.entity(), &entity))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Attribute>(e.attribute(), &attribute))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Value>(e.value(), &value))
            .collect::<HashSet<EntityAttributeValue>>())
    }
}

#[cfg(test)]
pub mod tests {
    use eav::memory::EavMemoryStorage;
    use holochain_core_types::cas::{
        content::{AddressableContent, ExampleAddressableContent},
        storage::EAVTestSuite,
    };

    #[test]
    fn memory_eav_round_trip() {
        let entity_content = ExampleAddressableContent::from_content(&"foo".to_string());
        let attribute = "favourite-color".to_string();
        let value_content = ExampleAddressableContent::from_content(&"blue".to_string());
        EAVTestSuite::test_round_trip_test(
            EavMemoryStorage::new(),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    fn memory_eav_one_to_many() {
        let eav_storage = EavMemoryStorage::new();
        EAVTestSuite::test_one_to_many::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

    #[test]
    fn memory_eav_many_to_one() {
        let eav_storage = EavMemoryStorage::new();
        EAVTestSuite::test_many_to_one::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

}
