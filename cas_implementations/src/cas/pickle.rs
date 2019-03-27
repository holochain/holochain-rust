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
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

const PERSISTENCE_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct PickleStorage {
    id: Uuid,
    db: Arc<PickleDb>,
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
            db: Arc::new(PickleDb::new(
                db_path,
                PickleDbDumpPolicy::PeriodicDump(PERSISTENCE_INTERVAL),
                SerializationMethod::Cbor,
            )),
        }
    }

    fn db_mut(&mut self) -> Result<&mut PickleDb, HolochainError> {
        Arc::get_mut(&mut self.db).ok_or_else(|| HolochainError::ErrorGeneric("SHIT".into()))
    }
}

impl ContentAddressableStorage for PickleStorage {
    fn add(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        let inner = self.db_mut()?;

        inner
            .set(&content.address().to_string(), &content.content())
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;

        Ok(())
    }

    fn contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(self.db.exists(&address.to_string()))
    }

    fn fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        Ok(None)
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}
