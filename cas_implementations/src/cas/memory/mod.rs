mod actor;
use actor::{AskSelf, Protocol};
use cas::memory::actor::MemoryStorageActor;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use riker::actors::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq)]
pub struct MemoryStorage {
    actor: ActorRef<Protocol>,
    id: Uuid,
}

impl MemoryStorage {
    pub fn new() -> Result<MemoryStorage, HolochainError> {
        Ok(MemoryStorage {
            actor: MemoryStorageActor::new_ref()?,
            id: Uuid::new_v4(),
        })
    }
}

impl ContentAddressableStorage for MemoryStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasAdd(content.address(), content.content()))?;

        match response {
            Protocol::CasAddResult(add_result) => add_result,
            _ => Err(
                HolochainError::ErrorGeneric(
                    format!("Expected Protocol::CasAddResult received {:?}", response)
                )
            ),
        }
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasContains(address.clone()))?;

        match response {
            Protocol::CasContainsResult(contains_result) => contains_result,
            _ => Err(
                HolochainError::ErrorGeneric(
                    format!("Expected Protocol::CasContainsResult received {:?}", response)
                )
            )
        }
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasFetch(address.clone()))?;

        match response {
            Protocol::CasFetchResult(fetch_result) => Ok(fetch_result?),
            _ => Err(
                HolochainError::ErrorGeneric(
                    format!("Expected Protocol::CasFetchResult received {:?}", response),
                )
            )
        }
    }

    fn get_id(&self) -> Uuid {
        self.id
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
