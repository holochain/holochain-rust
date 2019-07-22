use holochain_net::tweetlog::*;

use lib3h_protocol::data_types::{EntryAspectData, EntryData};

use holochain_persistence_api::cas::content::Address;
use std::collections::HashMap;

pub struct EntryStore {
    // TODO: Changed once meta is only Addresses
    // pub meta_store: HashMap<MetaKey, HashSet<Address>>,
    pub store: HashMap<Address, HashMap<Address, EntryAspectData>>,
}

impl EntryStore {
    pub fn new() -> Self {
        EntryStore {
            store: HashMap::new(),
        }
    }

    /// Check if this value is already stored
    pub fn has(&self, entry_address: &Address, aspect_address: &Address) -> bool {
        let maybe_map = self.store.get(entry_address);
        if maybe_map.is_none() {
            return false;
        }
        maybe_map.unwrap().get(&aspect_address).is_some()
    }

    ///
    pub fn insert_entry(&mut self, entry: &EntryData) {
        log_tt!(
            "entrystore", "EntryStore: adding content for '{:?}'",
            entry.entry_address
        );
        if self.store.get(&entry.entry_address).is_none() {
            let mut map = HashMap::new();
            log_tt!("entrystore", "  -> first content!");
            for aspect in entry.aspect_list.clone() {
                map.insert(aspect.aspect_address.clone(), aspect.clone());
            }
            self.store.insert(entry.entry_address.clone(), map);
            return;
        }
        if let Some(map) = self.store.get_mut(&entry.entry_address) {
            for aspect in entry.aspect_list.clone() {
                map.insert(aspect.aspect_address.clone(), aspect.clone());
            }
        }
    }

    ///
    pub fn insert_aspect(&mut self, entry_address: &Address, aspect: &EntryAspectData) {
        log_tt!(
            "entrystore", "EntryStore: adding content for '{}': {:?}",
            entry_address, aspect.aspect_address,
        );
        if self.store.get(&entry_address).is_none() {
            let mut map = HashMap::new();
            log_tt!("entrystore", "  -> first content!");
            map.insert(aspect.aspect_address.clone(), aspect.clone());
            self.store.insert(entry_address.clone(), map);
            return;
        }
        if let Some(map) = self.store.get_mut(&entry_address) {
            map.insert(aspect.aspect_address.clone(), aspect.clone());
        }
    }

    /// Get all values for a meta_key as a vec
    pub fn get(&self, entry_address: &Address) -> Option<EntryData> {
        let aspect_map = self.store.get(entry_address)?;
        let aspect_list: Vec<EntryAspectData> = aspect_map.iter().map(|(_, v)| v.clone()).collect();
        return if aspect_list.is_empty() {
            None
        } else {
            Some(EntryData {
                entry_address: entry_address.clone(),
                aspect_list,
            })
        };
    }

    /// Get all values for a meta_key as a vec
    pub fn get_aspect(
        &self,
        entry_address: &Address,
        aspect_address: &Address,
    ) -> Option<EntryAspectData> {
        let maybe_entry = self.get(entry_address);
        if maybe_entry.is_none() {
            return None;
        }
        return maybe_entry.unwrap().get(&aspect_address.clone());
    }

    //    /// Get all values stored
    //    pub fn get_all(&self) -> Vec<MetaTuple> {
    //        let mut meta_list: Vec<MetaTuple> = Vec::new();
    //        for (meta_key, meta_map) in self.store.clone() {
    //            for (_, v) in meta_map {
    //                meta_list.push((meta_key.0.clone(), meta_key.1.clone(), v));
    //            }
    //        }
    //        meta_list
    //    }
}
