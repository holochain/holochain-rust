use chrono::{offset::Utc, DateTime};
use holochain_core_types::{
    cas::content::AddressableContent,
    eav::{
        create_key, from_key, Action, Attribute, Entity, EntityAttributeValue,
        EntityAttributeValueStorage, Value,
    },
    error::HolochainError,
    hash::HashString,
};
use im::hashmap::HashMap;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EavMemoryStorage {
    storage: Arc<RwLock<HashMap<HashString, EntityAttributeValue>>>,
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
            storage: Arc::new(RwLock::new(HashMap::new())),
            id: Uuid::new_v4(),
        }
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        let mut map = self.storage.write()?;
        let key = create_key(Action::insert)?;
        map.insert(key, eav.clone());
        Ok(())
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<HashString, EntityAttributeValue>, HolochainError> {
        let map = self.storage.read()?.clone();
        Ok(map
            .into_iter()
            //.cloned()
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.entity(), entity.as_ref()))
            .filter(|(_, e)| {
                EntityAttributeValue::filter_on_eav(&e.attribute(), attribute.as_ref())
            })
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.value(), value.as_ref()))
            .collect::<HashMap<HashString, EntityAttributeValue>>())
    }

    fn fetch_eav_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashMap<HashString, EntityAttributeValue>, HolochainError> {
        let map = self.storage.read()?.clone();
        Ok(map
            .into_iter()
            .filter(|(k, _)| {
                start_date
                    .map(|start| {
                        let (unix_time, _) =
                            from_key(k.clone()).unwrap_or((i64::min_value(), Action::None));
                        unix_time > start.timestamp()
                    })
                    .unwrap_or(true)
            })
            .filter(|(k, _)| {
                end_date
                    .map(|end| {
                        let (unix_time, _) =
                            from_key(k.clone()).unwrap_or((i64::max_value(), Action::None));
                        unix_time < end.timestamp()
                    })
                    .unwrap_or(true)
            })
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.entity(), entity.as_ref()))
            .filter(|(_, e)| {
                EntityAttributeValue::filter_on_eav(&e.attribute(), attribute.as_ref())
            })
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.value(), value.as_ref()))
            .collect::<HashMap<HashString, EntityAttributeValue>>())
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
