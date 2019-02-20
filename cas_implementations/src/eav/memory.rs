use holochain_core_types::{
    eav::{
        increment_key_till_no_collision, EaviQuery, EntityAttributeValueIndex,
        EntityAttributeValueStorage,
    },
    error::HolochainError,
};
use std::{
    collections::BTreeSet,
    sync::{Arc, RwLock},
};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EavMemoryStorage {
    storage: Arc<RwLock<BTreeSet<EntityAttributeValueIndex>>>,
    id: Uuid,
}

impl PartialEq for EavMemoryStorage {
    fn eq(&self, other: &EavMemoryStorage) -> bool {
        self.id == other.id
    }
}

impl EavMemoryStorage {
    pub fn new() -> EavMemoryStorage {
        EavMemoryStorage {
            storage: Arc::new(RwLock::new(BTreeSet::new())),
            id: Uuid::new_v4(),
        }
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let mut map = self.storage.write()?;
        let new_eav = increment_key_till_no_collision(eav.clone(), map.clone())?;
        map.insert(new_eav.clone());
        Ok(Some(new_eav.clone()))
    }

    fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let map = self.storage.read()?;
        let iter = map.iter().cloned();
        Ok(query.run(iter))
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
        eav::Attribute,
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
            EavMemoryStorage::new(),
            entity_content,
            attribute,
            value_content,
        )
    }

    #[test]
    fn memory_eav_one_to_many() {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_one_to_many::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

    #[test]
    fn memory_eav_many_to_one() {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_many_to_one::<ExampleAddressableContent, EavMemoryStorage>(eav_storage)
    }

    #[test]
    fn example_eav_range() {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_range::<ExampleAddressableContent, EavMemoryStorage>(eav_storage);
    }

    #[test]
    fn file_eav_prefixes() {
        let eav_storage = EavMemoryStorage::new();
        EavTestSuite::test_multiple_attributes::<ExampleAddressableContent, EavMemoryStorage>(
            eav_storage,
            vec!["a_", "b_", "c_", "d_"]
                .into_iter()
                .map(|p| Attribute::LinkTag(p.to_string() + "one_to_many"))
                .collect(),
        );
    }

}
