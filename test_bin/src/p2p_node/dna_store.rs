use persistence_api::cas::content::Address;
use std::collections::HashMap;

use super::meta_store::MetaStore;

/// Holds DNA-specific data
pub struct DnaStore {
    pub dna_address: Address,
    pub entry_store: HashMap<Address, serde_json::Value>,
    pub meta_store: MetaStore,
    pub authored_entry_store: HashMap<Address, serde_json::Value>,
    pub authored_meta_store: MetaStore,
}

impl DnaStore {
    pub fn new(dna_address: Address) -> Self {
        DnaStore {
            dna_address,
            entry_store: HashMap::new(),
            meta_store: MetaStore::new(),
            authored_entry_store: HashMap::new(),
            authored_meta_store: MetaStore::new(),
        }
    }
}
