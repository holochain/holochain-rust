use cas::{
    content::{Address, AddressableContent, Content},
    storage::ContentAddressableStorage,
};
use error::HolochainError;
use std::collections::HashMap;

pub struct MemoryStorage {
    storage: HashMap<Address, Content>,
}

impl MemoryStorage {
    pub fn new() -> MemoryStorage {
        MemoryStorage {
            storage: HashMap::new(),
        }
    }
}

impl ContentAddressableStorage for MemoryStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        self.storage.insert(content.address(), content.content());
        Ok(())
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(self.storage.contains_key(address))
    }

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        Ok(self
            .storage
            .get(address)
            .and_then(|c| Some(C::from_content(c))))
    }
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::tests::{ExampleAddressableContent, OtherExampleAddressableContent},
        memory::MemoryStorage,
        storage::tests::StorageTestSuite,
    };

    #[test]
    fn memory_round_trip() {
        let test_suite = StorageTestSuite::new(MemoryStorage::new());
        test_suite.round_trip_test::<ExampleAddressableContent, OtherExampleAddressableContent>(
            String::from("foo"),
            String::from("bar"),
        );
    }

}
