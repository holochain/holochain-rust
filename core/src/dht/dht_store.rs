use hash_table::{
    sys_entry::ToEntry,
    links_entry::{Link, LinkEntry, LinkActionKind},
};
use cas::storage::ContentAddressableStorage;
use cas::content::Content;

// FIXME placeholder
pub type EntityAttributeValue = String;

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
pub struct DhtStore<CAS: ContentAddressableStorage> {
    // storage holding local shard data
    storage: CAS,
    // Placeholder network module
    network: Network,
}
impl<CAS: ContentAddressableStorage> DhtStore<CAS> {
    // Linking
    pub fn add_link(&mut self, link: &Link) {
        // FIXME
        self.storage.add(
            &LinkEntry::from_link(LinkActionKind::ADD, link).to_entry().content());
    }
    pub fn remove_link() {
        // FIXME
    }
    pub fn get_links() {
        // FIXME
    }

    // getters
    pub fn storage(&self) -> &CAS { &self.storage }
    pub fn network(&self) -> &Network { &self.network }
}
