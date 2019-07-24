use holochain_net::{
   tweetlog::*,
};

use lib3h_protocol::data_types::{MetaKey, MetaTuple};

use holochain_persistence_api::{cas::content::Address, hash::HashString};
use multihash::Hash;
use std::collections::HashMap;

pub type MetaStoreValue = serde_json::Value;

pub struct MetaStore {
    // TODO: Changed once meta is only Addresses
    // pub meta_store: HashMap<MetaKey, HashSet<Address>>,
    store: HashMap<MetaKey, HashMap<Address, serde_json::Value>>,
}

impl MetaStore {
    pub fn new() -> Self {
        MetaStore {
            store: HashMap::new(),
        }
    }

    /// Check if this value is already stored
    pub fn has(&self, meta_key: MetaKey, v: &MetaStoreValue) -> bool {
        let hash = HashString::encode_from_str(&v.to_string(), Hash::SHA2256);
        let maybe_map = self.store.get(&meta_key);
        if maybe_map.is_none() {
            return false;
        }
        maybe_map.unwrap().get(&hash).is_some()
    }

    ///
    pub fn insert(&mut self, meta_key: MetaKey, v: MetaStoreValue) {
        let hash = HashString::encode_from_str(&v.to_string(), Hash::SHA2256);
        if let None = self.store.get_mut(&meta_key) {
            let mut map = HashMap::new();
            log_tt!(
                "metastore",
                "MetaStore: first content for '{:?}' = {} | {}",
                meta_key,
                v,
                hash,
            );
            map.insert(hash, v);
            self.store.insert(meta_key, map);
        } else {
            if let Some(map) = self.store.get_mut(&meta_key) {
                //assert!(map.get(&hash).is_none());
                log_tt!(
                    "metastore",
                    "MetaStore: adding content for '{:?}' = {} | {}",
                    meta_key,
                    v,
                    hash,
                );
                map.insert(hash, v);
            };
        };
    }

    /// Get all values for a meta_key as a vec
    pub fn get(&self, meta_key: MetaKey) -> Vec<serde_json::Value> {
        let maybe_metas = self.store.get(&meta_key);
        let metas = match maybe_metas.clone() {
            Some(map) => map.clone(),
            // if meta not found return empty list (will make the aggregation easier)
            None => HashMap::new(),
        };
        let res = metas.iter().map(|(_, v)| v.clone()).collect();
        res
    }

    /// Get all values stored
    pub fn get_all(&self) -> Vec<MetaTuple> {
        let mut meta_list: Vec<MetaTuple> = Vec::new();
        for (meta_key, meta_map) in self.store.clone() {
            for (_, v) in meta_map {
                meta_list.push((meta_key.0.clone(), meta_key.1.clone(), v));
            }
        }
        meta_list
    }
}
