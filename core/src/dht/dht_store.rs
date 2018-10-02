use hash_table::{
    sys_entry::ToEntry,
    links_entry::{Link, LinkEntry, LinkActionKind},
};
use cas::{
    storage::ContentAddressableStorage,
    content::Content,
    eav::EntityAttributeValue,
    eav::EntityAttributeValueStorage,
};
use hash::HashString;
use std::collections::HashSet;
use error::HolochainError;

// Placeholder network module
#[derive(Clone, Debug, PartialEq)]
pub struct Network {
    // FIXME
}
impl Network {
    pub fn publish(&mut self, content: &Content) {
        // FIXME
    }
    pub fn publish_meta(&mut self, meta: &EntityAttributeValue) {
        // FIXME
    }
}


/// The state-slice for the DHT.
/// Holds the agent's local shard and interacts with the network module
#[derive(Clone, Debug, PartialEq)]
pub struct DhtStore<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage> {
    // storages holding local shard data
    data_storage: CAS,
    meta_storage: EAVS,
    // Placeholder network module
    network: Network,
}

impl<CAS: ContentAddressableStorage, EAVS: EntityAttributeValueStorage> DhtStore<CAS, EAVS> {
    // Linking
    pub fn add_link(&mut self, link: &Link) {
        // FIXME
        let link_entry = LinkEntry::from_link(LinkActionKind::ADD, link);
        self.data_storage.add(&link_entry.to_entry().content());
        self.meta_storage.add_eav(&link_entry.to_eav());
    }

    pub fn remove_link(&mut self) {
        // FIXME
    }

    pub fn get_links(&self, address: HashString, attribute_name: String) -> Result<HashSet<EntityAttributeValue>, HolochainError>{
        // FIXME query network?

        // FIXME get my own links
        let qres = self.meta_storage.fetch_eav(Some(address), Some(attribute_name), None);
        qres
    }

    // getters
    pub fn data_storage(&self) -> &CAS { &self.data_storage }
    pub fn meta_storage(&self) -> &EAVS { &self.meta_storage }
    pub fn network(&self) -> &Network { &self.network }
}
