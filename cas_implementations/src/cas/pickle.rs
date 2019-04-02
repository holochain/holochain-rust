use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    error::HolochainError,
};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::{
    fmt::{Debug, Error, Formatter},
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};
use uuid::Uuid;

const PERSISTENCE_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct PickleStorage {
    id: Uuid,
    db: Arc<RwLock<PickleDb>>,
}

impl Debug for PickleStorage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("PickleStorage")
            .field("id", &self.id)
            .finish()
    }
}

impl PickleStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> PickleStorage {
        PickleStorage {
            id: Uuid::new_v4(),
            db: Arc::new(RwLock::new(PickleDb::new(
                db_path,
                PickleDbDumpPolicy::PeriodicDump(PERSISTENCE_INTERVAL),
                SerializationMethod::Cbor,
            ))),
        }
    }
}

impl ContentAddressableStorage for PickleStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let mut inner = self.db.write()?;

        inner
            .set(&content.address().to_string(), &content.content())
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;

        Ok(())
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        let inner = self.db.read()?;

        Ok(inner.exists(&address.to_string()))
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        let inner = self.db.read()?;

        Ok(inner.get(&address.to_string()))
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::cas::pickle::PickleStorage;
    use holochain_core_types::{
        cas::{
            content::{ExampleAddressableContent, OtherExampleAddressableContent},
            storage::StorageTestSuite,
        },
        json::RawString,
    };
    use tempfile::{tempdir, TempDir};

    pub fn test_pickle_cas() -> (PickleStorage, TempDir) {
        let dir = tempdir().expect("Could not create a tempdir for CAS testing");
        (PickleStorage::new(dir.path()), dir)
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
