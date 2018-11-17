use action::ActionWrapper;
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    eav::{EntityAttributeValue, EntityAttributeValueStorage},
    error::HolochainError,
    link::Link,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

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
#[derive(Clone, Debug)]
pub struct DhtStore {
    // Storages holding local shard data
    content_storage: Arc<RwLock<ContentAddressableStorage>>,
    meta_storage: Arc<RwLock<EntityAttributeValueStorage>>,
    // Placeholder network module
    network: Network,

    add_link_actions: HashMap<ActionWrapper, Result<(), HolochainError>>,
}

impl PartialEq for DhtStore {
    fn eq(&self, other: &DhtStore) -> bool {
        let content = &self.content_storage.clone();
        let other_content = &other.content_storage().clone();
        let meta = &self.meta_storage.clone();
        let other_meta = &other.meta_storage.clone();

        self.network == other.network
            && self.add_link_actions == other.add_link_actions
            && (*content.read().unwrap()).get_id() == (*other_content.read().unwrap()).get_id()
            && *meta.read().unwrap() == *other_meta.read().unwrap()
    }
}

impl DhtStore {
    // LifeCycle
    // =========
    pub fn new(
        content_storage: Arc<RwLock<ContentAddressableStorage>>,
        meta_storage: Arc<RwLock<EntityAttributeValueStorage>>,
    ) -> Self {
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
        address: Address,
        tag: String,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        self.meta_storage.read().unwrap().fetch_eav(
            Some(address),
            Some(format!("link__{}", tag)),
            None,
        )
    }

    // Getters (for reducers)
    // =======
    pub(crate) fn content_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.content_storage.clone()
    }
    pub(crate) fn meta_storage(&self) -> Arc<RwLock<EntityAttributeValueStorage>> {
        self.meta_storage.clone()
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
    pub(crate) fn add_link_actions_mut(
        &mut self,
    ) -> &mut HashMap<ActionWrapper, Result<(), HolochainError>> {
        &mut self.add_link_actions
    }
}
