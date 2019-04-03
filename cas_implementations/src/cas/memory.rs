use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MemoryStorage {
    storage: Arc<RwLock<HashMap<Address, Content>>>,
    id: Uuid,
}

impl PartialEq for MemoryStorage {
    fn eq(&self, other: &MemoryStorage) -> bool {
        self.id == other.id
    }
}
impl Default for MemoryStorage {
    fn default() -> MemoryStorage {
        MemoryStorage {
            storage: Arc::new(RwLock::new(HashMap::new())),
            id: Uuid::new_v4(),
        }
    }
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage {
        Default::default()
    }
}

impl ContentAddressableStorage for MemoryStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let mut map = self.storage.write()?;
        map.insert(content.address().clone(), content.content().clone());
        Ok(())
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let map = self.storage.read()?;
        Ok(map.contains_key(address))
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        let map = self.storage.read()?;
        Ok(map.get(address).cloned())
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
pub mod tests {
    use crate::cas::memory::MemoryStorage;
    use holochain_core_types::{
        cas::{
            content::{ExampleAddressableContent, OtherExampleAddressableContent},
            storage::StorageTestSuite,
        },
        json::RawString,
    };

    pub fn test_memory_storage() -> MemoryStorage {
        MemoryStorage::new()
    }

    #[test]
    fn memory_round_trip() {
        let test_suite = StorageTestSuite::new(test_memory_storage());
        test_suite.round_trip_test::<ExampleAddressableContent, OtherExampleAddressableContent>(
            RawString::from("foo").into(),
            RawString::from("bar").into(),
        );
    }

}
