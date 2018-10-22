use action::ActionWrapper;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content}, storage::ContentAddressableStorage,
    },
    eav::{EntityAttributeValue, EntityAttributeValueStorage}, error::HolochainError,
    hash::HashString, links_entry::Link,
};
use std::collections::{HashMap, HashSet};

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

    pub fn get(&mut self, _address: &Address) -> Option<Content> {
        // FIXME
        None
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

    add_link_actions: HashMap<ActionWrapper, Result<(), HolochainError>>,
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
            add_link_actions: HashMap::new(),
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
    pub fn content_storage(&self) -> CAS {
        self.content_storage.clone()
    }
    pub(crate) fn content_storage_mut(&mut self) -> &mut CAS {
        &mut self.content_storage
    }
    pub fn meta_storage(&self) -> EAVS {
        self.meta_storage.clone()
    }
    pub(crate) fn meta_storage_mut(&mut self) -> &mut EAVS {
        &mut self.meta_storage
    }
    pub(crate) fn network(&self) -> &Network {
        &self.network
    }
    pub(crate) fn network_mut(&mut self) -> &mut Network {
        &mut self.network
    }
    pub fn add_link_actions(&self) -> &HashMap<ActionWrapper, Result<(), HolochainError>> {
        &self.add_link_actions
    }
    pub(crate) fn add_link_actions_mut(&mut self) -> &mut HashMap<ActionWrapper, Result<(), HolochainError>> {
        &mut self.add_link_actions
    }
}
