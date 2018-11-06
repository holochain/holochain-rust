use action::ActionWrapper;
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
use std::collections::{HashMap, HashSet};
use std::sync::{Arc,Mutex};

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
pub struct DhtStore<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    
{
    // Storages holding local shard data
    content_storage: CAS,
    meta_storage: Arc<Mutex<EntityAttributeValueStorage>>,
    // Placeholder network module
    network: Network,

    add_link_actions: HashMap<ActionWrapper, Result<(), HolochainError>>,
}

impl<CAS> PartialEq for DhtStore<CAS> where CAS: ContentAddressableStorage + Sized + Clone + PartialEq {
    fn eq(&self, other: &DhtStore<CAS>) -> bool {
        self.content_storage == other.content_storage &&
        self.network == other.network &&
        self.add_link_actions == other.add_link_actions &&
        *self.meta_storage.lock().unwrap() == *other.meta_storage.lock().unwrap()
    }
}

impl<CAS> DhtStore<CAS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    
{
    // LifeCycle
    // =========
    pub fn new(content_storage: CAS, meta_storage:Arc<Mutex<EntityAttributeValueStorage>>) -> Self {
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
        address: HashString,
        tag: String,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        self.meta_storage
            .lock().unwrap().fetch_eav(Some(address), Some(format!("link__{}", tag)), None)
    }

    // Getters (for reducers)
    // =======
    pub fn content_storage(&self) -> CAS {
        self.content_storage.clone()
    }
    pub(crate) fn content_storage_mut(&mut self) -> &mut CAS {
        &mut self.content_storage
    }
    pub(crate) fn meta_storage(&self) -> Arc<Mutex<EntityAttributeValueStorage>> {
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
