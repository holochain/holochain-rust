mod actor;
use actor::{AskSelf, Protocol};
use cas::memory::actor::MemoryStorageActor;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use riker::actors::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryStorage {
    actor: ActorRef<Protocol>,
}

impl MemoryStorage {
    pub fn new() -> Result<MemoryStorage, HolochainError> {
        Ok(MemoryStorage {
            actor: MemoryStorageActor::new_ref()?,
        })
    }
}

impl ContentAddressableStorage for MemoryStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasAdd(content.address(), content.content()))?;
        unwrap_to!(response => Protocol::CasAddResult).clone()
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasContains(address.clone()))?;
        unwrap_to!(response => Protocol::CasContainsResult).clone()
    }

    fn fetch<AC: AddressableContent>(
        &self,
        address: &Address,
    ) -> Result<Option<AC>, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasFetch(address.clone()))?;
        let content = unwrap_to!(response => Protocol::CasFetchResult).clone()?;
        Ok(content.and_then(|c| Some(AC::from_content(&c))))
    }
}

#[cfg(test)]
pub mod tests {
    use cas::memory::MemoryStorage;
    use holochain_core_types::{
        cas::{
            content::{ExampleAddressableContent, OtherExampleAddressableContent},
            storage::StorageTestSuite,
        },
        json::RawString,
    };

    pub fn test_memory_storage() -> MemoryStorage {
        MemoryStorage::new().expect("could not create memory storage")
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
