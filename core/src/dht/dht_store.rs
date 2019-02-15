use crate::{action::ActionWrapper, dht::dht_reducers::ENTRY_HEADER_ATTRIBUTE};
use holochain_core_types::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    chain_header::ChainHeader,
    eav::{
        Attribute, EavFilter, EaviQuery, EntityAttributeValueIndex, EntityAttributeValueStorage,
    },
    entry::Entry,
    error::HolochainError,
};

use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, RwLock},
};

/// The state-slice for the DHT.
/// Holds the agent's local shard and interacts with the network module
#[derive(Clone, Debug)]
pub struct DhtStore {
    // Storages holding local shard data
    content_storage: Arc<RwLock<ContentAddressableStorage>>,
    meta_storage: Arc<RwLock<EntityAttributeValueStorage>>,

    actions: HashMap<ActionWrapper, Result<Address, HolochainError>>,
}

impl PartialEq for DhtStore {
    fn eq(&self, other: &DhtStore) -> bool {
        let content = &self.content_storage.clone();
        let other_content = &other.content_storage().clone();
        let meta = &self.meta_storage.clone();
        let other_meta = &other.meta_storage.clone();

        self.actions == other.actions
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
        DhtStore {
            content_storage,
            meta_storage,
            actions: HashMap::new(),
        }
    }

    pub fn get_links(
        &self,
        address: Address,
        tag: String,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let filtered = self.meta_storage.read()?.fetch_eavi(&EaviQuery::new(
            Some(address).into(),
            EavFilter::<Attribute>::attribute_prefixes(
                vec!["link__", "removed_link__"],
                Some(&tag),
            ),
            None.into(),
            Default::default(),
        ))?;

        Ok(filtered
            .into_iter()
            .filter(|eav| eav.attribute().starts_with("link__"))
            .collect())
    }

    /// Get all headers for an entry by first looking in the DHT meta store
    /// for header addresses, then resolving them with the DHT CAS
    pub fn get_headers(&self, entry_address: Address) -> Result<Vec<ChainHeader>, HolochainError> {
        self.meta_storage()
            .read()
            .unwrap()
            // fetch all EAV references to chain headers for this entry
            .fetch_eavi(&EaviQuery::new(
                Some(entry_address).into(),
                Some(ENTRY_HEADER_ATTRIBUTE.to_string()).into(),
                None.into(),
                Default::default(),
            ))?
            .into_iter()
            // get the header addresses
            .map(|eavi| eavi.value())
            // fetch the header content from CAS
            .map(|a| self.content_storage().read().unwrap().fetch(&a))
            // rearrange
            .collect::<Result<Vec<Option<_>>, _>>()
            .map(|r| {
                r.into_iter()
                    // ignore None values
                    .flatten()
                    .map(|content| ChainHeader::try_from_content(&content))
                    .collect::<Result<Vec<_>, _>>()
            })?
    }

    /// Add an entry and header to the CAS and EAV, respectively
    pub fn add_header_for_entry(
        &self,
        entry: &Entry,
        header: &ChainHeader,
    ) -> Result<(), HolochainError> {
        let eavi = EntityAttributeValueIndex::new(
            &entry.address(),
            &ENTRY_HEADER_ATTRIBUTE.to_string(),
            &header.address(),
        )?;
        self.content_storage().write().unwrap().add(header)?;
        self.meta_storage().write().unwrap().add_eavi(&eavi)?;
        Ok(())
    }

    // Getters (for reducers)
    // =======
    pub(crate) fn content_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.content_storage.clone()
    }
    pub(crate) fn meta_storage(&self) -> Arc<RwLock<EntityAttributeValueStorage>> {
        self.meta_storage.clone()
    }
    pub fn actions(&self) -> &HashMap<ActionWrapper, Result<Address, HolochainError>> {
        &self.actions
    }
    pub(crate) fn actions_mut(
        &mut self,
    ) -> &mut HashMap<ActionWrapper, Result<Address, HolochainError>> {
        &mut self.actions
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use holochain_core_types::{
        cas::storage::ExampleContentAddressableStorage, chain_header::test_chain_header_with_sig,
        eav::ExampleEntityAttributeValueStorage, entry::test_entry,
    };

    #[test]
    fn get_headers_roundtrip() {
        let store = DhtStore::new(
            Arc::new(RwLock::new(
                ExampleContentAddressableStorage::new().unwrap(),
            )),
            Arc::new(RwLock::new(ExampleEntityAttributeValueStorage::new())),
        );
        let entry = test_entry();
        let header1 = test_chain_header_with_sig("sig1");
        let header2 = test_chain_header_with_sig("sig2");
        store.add_header_for_entry(&entry, &header1).unwrap();
        store.add_header_for_entry(&entry, &header2).unwrap();
        let headers = store.get_headers(entry.address()).unwrap();
        assert_eq!(headers, vec![header1, header2]);
    }
}
