use holochain_core_types::{
    eav::{
        get_latest, increment_key_till_no_collision, Attribute, Entity, EntityAttributeValueIndex,
        EntityAttributeValueStorage, IndexQuery, Value,
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
    fn add_eav(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let mut map = self.storage.write()?;
        let new_eav = increment_key_till_no_collision(eav.clone(), map.clone())?;
        map.insert(new_eav.clone());
        Ok(Some(new_eav.clone()))
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
        index_query: IndexQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let map = self.storage.read()?;
        Ok(map
            .clone()
            .into_iter()
            .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.entity(), entity.as_ref()))
            .filter(|e| {
                EntityAttributeValueIndex::filter_on_eav(&e.attribute(), attribute.as_ref())
            })
            .filter(|e| EntityAttributeValueIndex::filter_on_eav(&e.value(), value.as_ref()))
            .filter(|e| {
                index_query
                    .start_time()
                    .map(|start| start >= e.index())
                    .unwrap_or_else(|| {
                        let latest = get_latest(e.clone(), map.clone())
                            .unwrap_or(EntityAttributeValueIndex::default());
                        latest.index() == e.index()
                    })
            })
            .filter(|e| {
                index_query
                    .start_time()
                    .map(|end| end <= e.index())
                    .unwrap_or_else(|| {
                        let latest = get_latest(e.clone(), map.clone())
                            .unwrap_or(EntityAttributeValueIndex::default());
                        latest.index() == e.index()
                    })
            })
            .collect::<BTreeSet<EntityAttributeValueIndex>>())
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

}
