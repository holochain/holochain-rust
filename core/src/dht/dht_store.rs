use hash_table::{
    sys_entry::ToEntry,
    links_entry::{Link, LinkEntry, LinkActionKind},
};
use cas::{
    //RelatableContentStorage,
    content::{Address, Content, AddressableContent},
    eav::EntityAttributeValue,
    storage::ContentAddressableStorage,
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


//pub type Dht = HolographicStore<HashTable>;

/// The state-slice for the DHT.
/// Holds the agent's local shard and interacts with the network module
#[derive(Clone, Debug, PartialEq)]
pub struct DhtStore<C: ContentAddressableStorage + Sized, M: EntityAttributeValueStorage + Sized> {
    // storages holding local shard data
    content_storage: C,
    meta_storage: M,

    // TODO - Temp storage for things we received from the network but are not validated yet
    // temp_storage: T,
    // Placeholder network module
    network: Network,
}

impl<C: ContentAddressableStorage + Sized, M: EntityAttributeValueStorage + Sized> DhtStore<C, M> {

//    pub fn add_content(content: AddressableContent) {
//        // FIXME publish to network
//        self.content_storage.add(content);
//    }

    // ContentAddressableStorage
    // -------------------------
    fn add_content(&mut self, content: &AddressableContent) -> Result<(), HolochainError> {
        self.content_storage.add(content)
    }

    fn contains_content(&self, address: &Address) -> Result<bool, HolochainError> {
        self.content_storage.contains(address)
    }

    fn fetch_content<C: AddressableContent>(&self, address: &Address) -> Result<Option<C>, HolochainError> {
        self.content_storage.fetch(address)
    }

    pub fn new(content_storage : C, meta_storage: M) {
        let network = Network{};
        DhtStore { content_storage, meta_storage, network }
    }

    // EntityAttributeValueStorage
    // ---------------------------
    fn add_eav(&mut self, eav: &EntityAttributeValue) -> Result<(), HolochainError> {
        self.meta_storage.add_eav(eav)
    }

    fn fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        self.meta_storage.fetch_eav(entity, attribute, value)
    }

    // Linking
    // -------
    pub fn add_link(&mut self, link: &Link) {
        // FIXME
        let link_entry = LinkEntry::from_link(LinkActionKind::ADD, link);
        self.storage.add(&link_entry.to_entry().content());
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
    pub fn storage(&self) -> &T { &self.storage }
    // pub fn temp_storage(&self) -> &T { &self.temp_storage }
    pub fn network(&self) -> &Network { &self.network }
}
