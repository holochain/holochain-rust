use holochain_core_types::{
    eav::{
        create_key, Action, Attribute, Entity, EntityAttributeValue, EntityAttributeValueStorage,
        Key, Value,
    },
    error::HolochainError,
};
use im::ordmap::OrdMap;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EavMemoryStorage {
    storage: OrdMap<Key, EntityAttributeValue>,
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
            storage: OrdMap::new(),
            id: Uuid::new_v4(),
        }
    }
}

impl EntityAttributeValueStorage for EavMemoryStorage {
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        if self
            .fetch_eav(Some(eav.entity()), Some(eav.attribute()), Some(eav.value()))?
            .len()
            == 0
        {
            let map = &mut self.storage;
            let key = create_key(Action::Insert)?;
            map.insert(key, eav.clone());
            Ok(())
        } else {
            Ok(())
        }
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<OrdMap<Key, EntityAttributeValue>, HolochainError> {
        let map = &self.storage;
        let filtered_map = map
            .clone()
            .into_iter()
            //.cloned()
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.entity(), entity.as_ref()))
            .filter(|(_, e)| {
                EntityAttributeValue::filter_on_eav(&e.attribute(), attribute.as_ref())
            })
            .filter(|(_, e)| EntityAttributeValue::filter_on_eav(&e.value(), value.as_ref()))
            .collect::<OrdMap<Key, EntityAttributeValue>>();
        Ok(filtered_map)
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
