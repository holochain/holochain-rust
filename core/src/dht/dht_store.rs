use crate::action::ActionWrapper;
use holochain_core_types::{
    chain_header::ChainHeader,
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex},
    entry::Entry,
    error::HolochainError,
};
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::{EavFilter, EntityAttributeValueStorage, IndexFilter},
};
use regex::Regex;

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
    meta_storage: Arc<RwLock<EntityAttributeValueStorage<Attribute>>>,

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

pub fn create_get_links_eavi_query<'a>(
    address: Address,
    link_type: String,
    tag: String,
) -> Result<EaviQuery<'a>, HolochainError> {
    let link_type_regex = Regex::new(&link_type)
        .map_err(|_| HolochainError::from("Invalid regex passed for type"))?;
    let tag_regex =
        Regex::new(&tag).map_err(|_| HolochainError::from("Invalid regex passed for tag"))?;
    Ok(EaviQuery::new(
        Some(address).into(),
        EavFilter::predicate(move |attr: Attribute| match attr.clone() {
            Attribute::LinkTag(query_link_type, query_tag)
            | Attribute::RemovedLink(query_link_type, query_tag) => {
                link_type_regex.is_match(&query_link_type) && tag_regex.is_match(&query_tag)
            }
            _ => false,
        }),
        None.into(),
        IndexFilter::LatestByAttribute,
        Some(EavFilter::single(Attribute::RemovedLink(
            link_type.clone(),
            tag.clone(),
        ))),
    ))
}

impl DhtStore {
    // LifeCycle
    // =========
    pub fn new(
        content_storage: Arc<RwLock<ContentAddressableStorage>>,
        meta_storage: Arc<RwLock<EntityAttributeValueStorage<Attribute>>>,
    ) -> Self {
        DhtStore {
            content_storage,
            meta_storage,
            actions: HashMap::new(),
        }
    }
    ///This algorithmn works by querying the EAVI Query for entries that match the address given, the link _type given, the tag given and a tombstone query set of RemovedLink(link_type,tag)
    ///this means no matter how many links are added after one is removed, we will always say that the link has been removed.
    ///One thing to remember is that LinkAdd entries occupy the "Value" aspect of our EAVI link stores.
    ///When that set is obtained, we filter based on the LinkTag attributes to only get the "live" links or links that are valid.
    pub fn get_links(
        &self,
        address: Address,
        link_type: String,
        tag: String,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let get_links_query = create_get_links_eavi_query(address, link_type, tag)?;
        let filtered = self.meta_storage.read()?.fetch_eavi(&get_links_query)?;
        Ok(filtered
            .into_iter()
            .filter(|eav| match eav.attribute() {
                Attribute::LinkTag(_, _) => true,
                _ => false,
            })
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
                Some(Attribute::EntryHeader).into(),
                None.into(),
                IndexFilter::LatestByAttribute,
                None,
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
            .map_err(|err| {
                let hc_error: HolochainError = err.into();
                hc_error
            })
    }

    /// Add an entry and header to the CAS and EAV, respectively
    pub fn add_header_for_entry(
        &self,
        entry: &Entry,
        header: &ChainHeader,
    ) -> Result<(), HolochainError> {
        let eavi = EntityAttributeValueIndex::new(
            &entry.address(),
            &Attribute::EntryHeader,
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
    pub(crate) fn meta_storage(&self) -> Arc<RwLock<EntityAttributeValueStorage<Attribute>>> {
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
    use holochain_core_types::{chain_header::test_chain_header_with_sig, entry::test_entry};

    use holochain_persistence_api::{
        cas::storage::ExampleContentAddressableStorage, eav::ExampleEntityAttributeValueStorage,
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
