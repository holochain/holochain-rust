use holochain_core_types::{
    eav::{EaviQuery, EntityAttributeValueIndex, EntityAttributeValueStorage},
    error::HolochainError,
};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::{
    collections::BTreeSet,
    fmt::{Debug, Error, Formatter},
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};
use uuid::Uuid;

const PERSISTENCE_INTERVAL: Duration = Duration::from_millis(5000);

#[derive(Clone)]
pub struct EavPickleStorage {
    db: Arc<RwLock<PickleDb>>,
    id: Uuid,
}

impl EavPickleStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> EavPickleStorage {
        EavPickleStorage {
            id: Uuid::new_v4(),
            db: Arc::new(RwLock::new(PickleDb::new(
                db_path,
                PickleDbDumpPolicy::PeriodicDump(PERSISTENCE_INTERVAL),
                SerializationMethod::Cbor,
            ))),
        }
    }
}

impl Debug for EavPickleStorage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("EavPickleStorage")
            .field("id", &self.id)
            .finish()
    }
}

impl EntityAttributeValueStorage for EavPickleStorage {
    fn add_eavi(
        &mut self,
        eav: &EntityAttributeValueIndex,
    ) -> Result<Option<EntityAttributeValueIndex>, HolochainError> {
        let mut inner = self.db.write()?;

        let eav_str = format!("{:?}", eav);

        inner
            .set(&eav_str, &())
            .map_err(|e| HolochainError::ErrorGeneric(e.to_string()))?;

        Ok(None)
    }

    fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let inner = self.db.read()?;

        let all_entrys = inner.get_all();

        Ok(all_entrys.into())
    }
}
