use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    eav::{EntityAttributeValue, EntityAttributeValueStorage},
    error::HolochainError,
    hash::HashString,
    links_entry::Link,
};
use std::collections::HashSet;

// Placeholder network module
#[derive(Clone, Debug, PartialEq)]
pub struct Network {
    // FIXME
}
impl Network {
    pub fn publish(&mut self, _content: &AddressableContent) {
        // FIXME
    }
    pub fn publish_meta(&mut self, _meta: &EntityAttributeValue) {
        // FIXME
    }

    pub fn get(&mut self, _address: &Address) -> Content {
        // FIXME
        AddressableContent::from_content(&"".to_string())
    }
}

/// The state-slice for the DHT.
/// Holds the agent's local shard and interacts with the network module
#[derive(Clone, Debug, PartialEq)]
pub struct DhtStore<CAS, EAVS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // Storages holding local shard data
    content_storage: CAS,
    meta_storage: EAVS,
    // Placeholder network module
    network: Network,
}

impl<CAS, EAVS> DhtStore<CAS, EAVS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    // LifeCycle
    // =========
    pub fn new(content_storage: CAS, meta_storage: EAVS) -> Self {
        let network = Network {};
        DhtStore {
            content_storage,
            meta_storage,
            network,
        }
    }

    // Linking
    // =======
    pub fn add_link(&mut self, _link: &Link) -> Result<(), HolochainError> {
        // FIXME
        Err(HolochainError::NotImplemented)
    }

    pub fn remove_link(&mut self) {
        // FIXME
    }

    pub fn get_links(
        &self,
        _address: HashString,
        _attribute_name: String,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        // FIXME
        Err(HolochainError::NotImplemented)
    }

    // Getters (for reducers)
    // =======
    pub(crate) fn content_storage(&self) -> CAS {
        self.content_storage.clone()
    }
    pub(crate) fn content_storage_mut(&mut self) -> &mut CAS {
        &mut self.content_storage
    }
    pub(crate) fn network(&self) -> &Network {
        &self.network
    }
    pub(crate) fn network_mut(&mut self) -> &mut Network {
        &mut self.network
    }
}
