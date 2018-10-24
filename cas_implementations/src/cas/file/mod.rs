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
use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};

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

struct FileVisitor;
impl<'de> Visitor<'de> for FileVisitor
{
    // The type that our Visitor is going to produce.
    type Value = FilesystemStorage;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<FilesystemStorage, M::Error>
    where
        M: MapAccess<'de>,
    {

        // While there are entries remaining in the input, add them
        // into our map.
        let key : (String,String) = access.next_entry()?.expect("Supposed to get first entry");
        Ok(FilesystemStorage::new(&key.1).expect("cannot create file"))
    }
}

impl<'de> Deserialize<'de> for FilesystemStorage
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(FileVisitor)
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
    extern crate serde_test;
    use serde_json;
    use self::serde_test::{Token, assert_tokens};

    use self::tempfile::{tempdir, TempDir};
    use cas::file::FilesystemStorage;
    use holochain_core_types::cas::{
        content::{ExampleAddressableContent, OtherExampleAddressableContent},
        storage::StorageTestSuite,
    };

    #[test]
    pub fn serialization_round_trip()
    {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let storage = FilesystemStorage::new(path).unwrap();
        let file_json = serde_json::to_string(&storage).unwrap();
        let file_serde : FilesystemStorage = serde_json::from_str(&file_json).unwrap();
        assert_eq!(file_serde.dir_path,storage.dir_path);
    }

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
