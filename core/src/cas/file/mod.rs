pub mod actor;

use cas::{
    content::{Address, AddressableContent},
    storage::ContentAddressableStorage,
};
use error::HolochainError;
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, MAIN_SEPARATOR},
};
use cas::file::actor::FilesystemStorageActor;
use actor::Protocol;
use riker::actors::*;
use cas::content::Content;

pub struct FilesystemStorage {
    dir_actor: ActorRef<Protocol>,
}

impl FilesystemStorage {
    pub fn new(dir_path: &str) -> Result<FilesystemStorage, HolochainError> {
        FilesystemStorage {
            dir_actor: FilesystemStorageActor::new_ref(dir_path)?,
        }
    }

    /// builds an absolute path for an AddressableContent address
    fn address_to_path(&self, address: &Address) -> String {
        // using .txt extension because content is arbitrary and controlled by the
        // AddressableContent trait implementation
        format!("{}{}{}.txt", self.dir_path, MAIN_SEPARATOR, address)
    }
}

impl Actor for FilesystemStorage {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        sender
            .try_tell(
                match message {
                    Protocol::CasAdd(address, content) => {
                        Protocol::CasAddResult(self.unsafe_add(address, content))
                    },
                    Protocol::CasContains(address) => {
                        Protocol::CasContainsResult(self.unsafe_contains(address))
                    },
                    Protocol::CasFetch(address) => {
                        Protocol::CasFetchResult(self.unsafe_fetch(address))
                    },
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell FilesystemStorage sender");
    }

}

impl ContentAddressableStorage for FilesystemStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let response = self.block_on_ask(Protocol::CasAdd(content.address(), content.content()))?;
        unwrap_to!(response => Protocol::CasAddResult).clone()
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let response = self.block_on_ask(Protocol::CasContains(address))?;
        unwrap_to!(response => Protocol::CasContainsResult).clone()
    }

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        let response = self.block_on_as(Protocol::CasFetch(address))?;
        unwrap_to!(response => Protocol::CasFetchResult).clone()
    }
}

#[cfg(test)]
pub mod tests {
    use cas::{
        content::tests::{ExampleAddressableContent, OtherExampleAddressableContent},
        file::FilesystemStorage,
        storage::tests::StorageTestSuite,
    };
    use tempfile::{tempdir, TempDir};

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
            String::from("foo"),
            String::from("bar"),
        );
    }

}
