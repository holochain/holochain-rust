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
use std::fmt;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor};

#[derive(Clone, PartialEq, Debug)]
pub struct FilesystemStorage {
    actor: ActorRef<Protocol>,
    dir_path : String,
}

impl FilesystemStorage {
    pub fn new(path: &str) -> Result<FilesystemStorage, HolochainError> {
        Ok(FilesystemStorage {
            actor: FilesystemStorageActor::new_ref(path)?,
            dir_path : String::from(path)
        })
    }

    pub fn dir_path(self) ->String
    {
        self.dir_path
    }
}

impl Serialize for FilesystemStorage
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("FilesystemStorage", 1)?;
        state.serialize_field("dir_path", &self.dir_path)?;
        state.end()
    }
}


        impl<'de> Deserialize<'de> for FilesystemStorage {
            fn deserialize<D>(deserializer: D) -> Result<FilesystemStorage, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FileVisitor;

                impl<'de> Visitor<'de> for FileVisitor {
                    type Value = FilesystemStorage;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<FilesystemStorage, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "dir_path" => Ok(FilesystemStorage::new(value).unwrap()),
                            _ => Err(de::Error::unknown_field(value, &["dir_path"])),
                        }
                    }
                }

                deserializer.deserialize_identifier(FileVisitor)
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
        let content = unwrap_to!(response => Protocol::CasFetchResult).clone()?;
        Ok(match content {
            Some(c) => Some(AC::from_content(&c)),
            None => None,
        })
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;

    use self::tempfile::{tempdir, TempDir};
    use cas::file::FilesystemStorage;
    use holochain_core_types::cas::{
        content::{ExampleAddressableContent, OtherExampleAddressableContent},
        storage::StorageTestSuite,
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
            String::from("foo"),
            String::from("bar"),
        );
    }

}
