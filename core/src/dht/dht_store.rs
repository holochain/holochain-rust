use cas::{
    content::{Address, AddressableContent, Content},
    storage::ContentAddressableStorage,
};
use eav::{EntityAttributeValue, EntityAttributeValueStorage};
// use eav::{Attribute, Entity, Value};
use error::HolochainError;
use hash::HashString;
use hash_table::{
    links_entry::{Link, LinkActionKind, LinkEntry},
    sys_entry::ToEntry,
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
    // storages holding local shard data
    pub(crate) content_storage: CAS,
    pub(crate) meta_storage: EAVS,

    // TODO - Temp storage for things we received from the network but are not validated yet
    // temp_storage: T,
    // Placeholder network module
    pub(crate) network: Network,
}

impl<CAS, EAVS> DhtStore<CAS, EAVS>
where
    CAS: ContentAddressableStorage + Sized + Clone + PartialEq,
    EAVS: EntityAttributeValueStorage + Sized + Clone + PartialEq,
{
    //    pub fn clone(&self) -> Self {
    //        DhtStore {
    //            content_storage: self.content_storage.clone(),
    //            meta_storage: self.meta_storage.clone(),
    //            network: self.network.clone(),
    //        }
    //    }

    // LifeCycle
    // ---------
    pub fn new(content_storage: CAS, meta_storage: EAVS) -> Self {
        let network = Network {};
        DhtStore {
            content_storage,
            meta_storage,
            network,
        }
    }
//
//    // ContentAddressableStorage
//    // -------------------------
//    fn add_content(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
//        // FIXME publish to network?
//        self.content_storage.add(content)
//    }
//
//    fn contains_content(&self, address: &Address) -> Result<bool, HolochainError> {
//        self.content_storage.contains(address)
//    }
//
//    fn fetch_content<C: AddressableContent>(
//        &self,
//        address: &Address,
//    ) -> Result<Option<C>, HolochainError> {
//        self.content_storage.fetch(address)
//    }
//
//    // EntityAttributeValueStorage
//    // ---------------------------
//    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
//        self.meta_storage.add_eav(eav)
//    }
//
//    fn fetch_eav(
//        &self,
//        entity: Option<Entity>,
//        attribute: Option<Attribute>,
//        value: Option<Value>,
//    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
//        self.meta_storage.fetch_eav(entity, attribute, value)
//    }
//
    // Linking
    // -------
    pub fn add_link(&mut self, link: &Link) -> Result<(), HolochainError> {
        // FIXME
        let link_entry = LinkEntry::from_link(LinkActionKind::ADD, link);
        self.content_storage.add(&link_entry.to_entry().1)
        // self.meta_storage.add_eav(&link_entry.to_eav());
    }

    pub fn remove_link(&mut self) {
        // FIXME
    }

    pub fn get_links(
        &self,
        address: HashString,
        attribute_name: String,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        // FIXME query network?

        // FIXME get my own links
        let qres = self
            .meta_storage
            .fetch_eav(Some(address), Some(attribute_name), None);
        qres
    }

    // getters (for reducers)
    pub(crate) fn content_storage(&self) -> &CAS {
        &self.content_storage
    }
    pub(crate) fn meta_storage(&self) -> &EAVS {
        &self.meta_storage
    }
    // pub fn temp_storage(&self) -> &T { &self.temp_storage }
    pub(crate) fn network(&self) -> &Network {
        &self.network
    }
}
