pub mod actor;
use actor::{AskSelf, Protocol};
use cas::file::actor::FilesystemStorageActor;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use riker::actors::*;

#[derive(Clone, PartialEq, Debug)]
pub struct FilesystemStorage {
    actor: ActorRef<Protocol>,
}

impl FilesystemStorage {
    pub fn new(path: &str) -> Result<FilesystemStorage, HolochainError> {
        Ok(FilesystemStorage {
            actor: FilesystemStorageActor::new_ref(path)?,
        })
    }
}

impl ContentAddressableStorage for FilesystemStorage {
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
        let maybe_content = unwrap_to!(response => Protocol::CasFetchResult).clone()?;

        match maybe_content {
            Some(content) => Ok(Some(AC::try_from_content(&content)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate serde_test;
    extern crate tempfile;

    use self::tempfile::{tempdir, TempDir};
    use cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::{
            content::{ExampleAddressableContent, OtherExampleAddressableContent},
            storage::StorageTestSuite,
        },
        json::RawString,
    };

    pub fn test_file_cas() -> (FilesystemStorage, TempDir) {
        let dir = tempdir().unwrap();
        (
            FilesystemStorage::new(dir.path().to_str().unwrap()).unwrap(),
            dir,
        )
    }

    #[test]
    /// show that content of different types can round trip through the same storage
    /// this is copied straight from the example with a file CAS
    fn file_content_round_trip_test() {
        let (cas, _dir) = test_file_cas();
        let test_suite = StorageTestSuite::new(cas);
        test_suite.round_trip_test::<ExampleAddressableContent, OtherExampleAddressableContent>(
            RawString::from("foo").into(),
            RawString::from("bar").into(),
        );
    }

}
