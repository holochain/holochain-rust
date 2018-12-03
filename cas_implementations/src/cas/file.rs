use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, MAIN_SEPARATOR},
    sync::{Arc, RwLock},
};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct FilesystemStorage {
    /// path to the directory where content will be saved to disk
    dir_path: String,
    id: Uuid,
    lock: Arc<RwLock<()>>,
}

impl PartialEq for FilesystemStorage {
    fn eq(&self, other: &FilesystemStorage) -> bool {
        self.id == other.id
    }
}

impl FilesystemStorage {
    pub fn new(dir_path: &str) -> Result<FilesystemStorage, HolochainError> {
        Ok(FilesystemStorage {
            dir_path: String::from(dir_path),
            id: Uuid::new_v4(),
            lock: Arc::new(RwLock::new(())),
        })
    }

    /// builds an absolute path for an AddressableContent address
    fn address_to_path(&self, address: &Address) -> String {
        // using .txt extension because content is arbitrary and controlled by the
        // AddressableContent trait implementation
        format!("{}{}{}.txt", self.dir_path, MAIN_SEPARATOR, address)
    }
}

impl ContentAddressableStorage for FilesystemStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let _guard = self.lock.write()?;
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&self.dir_path)?;

        write(
            self.address_to_path(&content.address()),
            content.content().to_string(),
        )?;

        Ok(())
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let _guard = self.lock.read()?;
        Ok(Path::new(&self.address_to_path(address)).is_file())
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        let _guard = self.lock.read()?;
        if self.contains(&address)? {
            Ok(Some(read_to_string(self.address_to_path(address))?.into()))
        } else {
            Ok(None)
        }
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
    use crate::cas::file::FilesystemStorage;
    use holochain_core_types::{
        cas::{
            content::{ExampleAddressableContent, OtherExampleAddressableContent},
            storage::StorageTestSuite,
        },
        json::RawString,
    };

    pub fn test_file_cas() -> (FilesystemStorage, TempDir) {
        let dir = tempdir().expect("Could not create a tempdir for CAS testing");
        (
            FilesystemStorage::new(&dir.path().to_string_lossy()).unwrap(),
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
