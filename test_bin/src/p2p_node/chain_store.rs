use super::entry_store::EntryStore;
use holochain_core_types::cas::content::Address;
use holochain_net::connection::json_protocol::EntryData;

/// Holds DNA-specific data
pub struct ChainStore {
    pub dna_address: Address,
    pub stored_entry_store: EntryStore,
    pub authored_entry_store: EntryStore,
}

impl ChainStore {
    pub fn new(dna_address: &Address) -> Self {
        ChainStore {
            dna_address: dna_address.clone(),
            stored_entry_store: EntryStore::new(),
            authored_entry_store: EntryStore::new(),
        }
    }

    pub fn get_entry(&self, entry_address: &Address) -> Option<EntryData> {
        let mut has_aspects = false;
        let mut entry = EntryData {
            entry_address: entry_address.clone(),
            aspect_list: vec![],
        };
        // Append what we have in `authored_entry_store`
        let maybe_entry_store = self.authored_entry_store.get(&entry_address);
        if let Some(mut local_entry) = maybe_entry_store {
            entry.aspect_list.append(&mut local_entry.aspect_list);
            has_aspects = true;
        }
        // Append what we have in `stored_entry_store`
        let maybe_entry_store = self.stored_entry_store.get(&entry_address);
        if let Some(mut local_entry) = maybe_entry_store {
            entry.aspect_list.append(&mut local_entry.aspect_list);
            has_aspects = true;
        }
        // Done
        return if has_aspects { Some(entry) } else { None };
    }
}
