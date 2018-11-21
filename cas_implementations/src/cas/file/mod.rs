pub mod actor;
use actor::{AskSelf, Protocol};
use cas::file::actor::FilesystemStorageActor;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use riker::actors::*;

use uuid::Uuid;

#[derive(Clone, PartialEq, Debug)]
pub struct FilesystemStorage {
    actor: ActorRef<Protocol>,
    id: Uuid,
}

impl FilesystemStorage {
    pub fn new(path: &str) -> Result<FilesystemStorage, HolochainError> {
        Ok(FilesystemStorage {
            actor: FilesystemStorageActor::new_ref(path)?,
            id: Uuid::new_v4(),
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
        println!("FilesystemStorage.contains({}) ?", address);
        let response = self
            .actor
            .block_on_ask(Protocol::CasContains(address.clone()))?;
        unwrap_to!(response => Protocol::CasContainsResult).clone()
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        let response = self
            .actor
            .block_on_ask(Protocol::CasFetch(address.clone()))?;

        Ok(unwrap_to!(response => Protocol::CasFetchResult).clone()?)
    }

    fn get_id(&self) -> Uuid {
        self.id
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
