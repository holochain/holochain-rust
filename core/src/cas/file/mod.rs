use cas::{
    content::{Address, AddressableContent},
    storage::ContentAddressableStorage,
};
use error::HolochainError;
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, MAIN_SEPARATOR},
};

pub struct FilesystemStorage {
    /// path to the directory where content will be saved to disk
    dir_path: String,
}

impl FilesystemStorage {
    pub fn new(dir_path: &str) -> Result<FilesystemStorage, HolochainError> {
        let canonical = Path::new(dir_path).canonicalize()?;
        if !canonical.is_dir() {
            return Err(HolochainError::IoError(
                "path is not a directory or permissions don't allow access".to_string(),
            ));
        }
        Ok(FilesystemStorage {
            dir_path: canonical
                .to_str()
                .ok_or_else(|| {
                    HolochainError::IoError("could not convert path to string".to_string())
                })?
                .to_string(),
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
        // @TODO be more efficient here
        // @see https://github.com/holochain/holochain-rust/issues/248
        create_dir_all(&self.dir_path)?;
        Ok(write(
            self.address_to_path(&content.address()),
            content.content(),
        )?)
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(Path::new(&self.address_to_path(address)).is_file())
    }

    fn fetch<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        if self.contains(&address)? {
            Ok(Some(C::from_content(&read_to_string(
                self.address_to_path(address),
            )?)))
        } else {
            Ok(None)
        }
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
