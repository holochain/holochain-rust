use crate::{
    action::ActionWrapper,
    content_store::{AddContent, GetContent},
};
use holochain_core_types::{
    chain_header::ChainHeader,
    crud_status::CrudStatus,
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex},
    entry::Entry,
    error::{HcResult, HolochainError},
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_locksmith::RwLock;
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent, Content},
        storage::ContentAddressableStorage,
    },
    eav::{EavFilter, EntityAttributeValueStorage, IndexFilter},
};
use regex::Regex;

use crate::{dht::pending_validations::PendingValidation, state::StateWrapper};
use holochain_json_api::error::JsonResult;
use holochain_persistence_api::error::PersistenceResult;
use lib3h_protocol::types::{AspectHash, EntryHash};
use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    convert::TryFrom,
    sync::Arc,
    time::{Duration, SystemTime},
};

/// The state-slice for the DHT.
/// Holds the CAS and EAVi that's used for the agent's local shard
/// as well as the holding list, i.e. list of all entries held for the DHT.
#[derive(Clone, Debug)]
pub struct DhtStore {
    // Storages holding local shard data
    content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    meta_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,

    /// All the entry aspects that the network has told us to hold
    holding_map: HashMap<EntryHash, HashSet<AspectHash>>,

    pub(crate) queued_holding_workflows:
        VecDeque<(PendingValidation, Option<(SystemTime, Duration)>)>,

    actions: HashMap<ActionWrapper, Result<Address, HolochainError>>,
}

impl PartialEq for DhtStore {
    fn eq(&self, other: &DhtStore) -> bool {
        let content = &self.content_storage.clone();
        let other_content = &other.content_storage.clone();
        let meta = &self.meta_storage.clone();
        let other_meta = &other.meta_storage.clone();

        self.actions == other.actions
            && (*content.read().unwrap()).get_id() == (*other_content.read().unwrap()).get_id()
            && *meta.read().unwrap() == *other_meta.read().unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct DhtStoreSnapshot {
    pub holding_list: Vec<Address>,
    pub queued_holding_workflows: VecDeque<(PendingValidation, Option<(SystemTime, Duration)>)>,
}

impl From<&StateWrapper> for DhtStoreSnapshot {
    fn from(state: &StateWrapper) -> Self {
        DhtStoreSnapshot {
            holding_list: state.dht().holding_list.clone(),
            queued_holding_workflows: state.dht().queued_holding_workflows.clone(),
        }
    }
}

pub static DHT_STORE_SNAPSHOT_ADDRESS: &str = "DhtStore";
impl AddressableContent for DhtStoreSnapshot {
    fn content(&self) -> Content {
        self.to_owned().into()
    }

    fn try_from_content(content: &Content) -> JsonResult<Self> {
        Self::try_from(content.to_owned())
    }

    fn address(&self) -> Address {
        DHT_STORE_SNAPSHOT_ADDRESS.into()
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
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        meta_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
    ) -> Self {
        DhtStore {
            content_storage,
            meta_storage,
            holding_list: Vec::new(),
            actions: HashMap::new(),
            queued_holding_workflows: VecDeque::new(),
        }
    }

    pub fn new_from_snapshot(
        content_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        meta_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
        snapshot: DhtStoreSnapshot,
    ) -> Self {
        let mut new_dht_store = Self::new(content_storage, meta_storage);
        new_dht_store.holding_list = snapshot.holding_list;
        new_dht_store.queued_holding_workflows = snapshot.queued_holding_workflows;
        new_dht_store
    }

    ///This algorithmn works by querying the EAVI Query for entries that match the address given, the link _type given, the tag given and a tombstone query set of RemovedLink(link_type,tag)
    ///this means no matter how many links are added after one is removed, we will always say that the link has been removed.
    ///One thing to remember is that LinkAdd entries occupy the "Value" aspect of our EAVI link stores.
    ///When that set is obtained, we filter based on the LinkTag and RemovedLink attributes to evaluate if they are "live" or "deleted". A reminder that links cannot be modified
    pub fn get_links(
        &self,
        address: Address,
        link_type: String,
        tag: String,
        crud_filter: Option<CrudStatus>,
    ) -> Result<BTreeSet<(EntityAttributeValueIndex, CrudStatus)>, HolochainError> {
        let get_links_query = create_get_links_eavi_query(address, link_type, tag)?;
        let filtered = self.meta_storage.read()?.fetch_eavi(&get_links_query)?;
        Ok(filtered
            .into_iter()
            .map(|s| match s.attribute() {
                Attribute::LinkTag(_, _) => (s, CrudStatus::Live),
                _ => (s, CrudStatus::Deleted),
            })
            .filter(|link_crud| crud_filter.map(|crud| crud == link_crud.1).unwrap_or(true))
            .collect())
    }

    pub fn get_all_metas(
        &self,
        address: &Address,
    ) -> Result<BTreeSet<EntityAttributeValueIndex>, HolochainError> {
        let query = EaviQuery::new(
            Some(address.to_owned()).into(),
            EavFilter::predicate(move |attr: Attribute| match attr.clone() {
                Attribute::LinkTag(_, _)
                | Attribute::RemovedLink(_, _)
                | Attribute::CrudLink
                | Attribute::CrudStatus => true,
                _ => false,
            }),
            None.into(),
            IndexFilter::LatestByAttribute,
            None,
        );
        Ok(self.meta_storage.read()?.fetch_eavi(&query)?)
    }

    /// Get all headers for an entry by first looking in the DHT meta store
    /// for header addresses, then resolving them with the DHT CAS
    pub fn get_headers(&self, entry_address: Address) -> Result<Vec<ChainHeader>, HolochainError> {
        self.meta_storage
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
            .map(|address| self.get(&address))
            // rearrange
            .collect::<Result<Vec<Option<_>>, _>>()
            .map(|r| {
                r.into_iter()
                    // ignore None values
                    .flatten()
                    .map(|entry| match entry {
                        Entry::ChainHeader(chain_header) => Ok(chain_header),
                        _ => Err(HolochainError::ErrorGeneric(
                            "Unexpected non-chain_header entry".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()
            })?
            .map_err(|err| {
                let hc_error: HolochainError = err;
                hc_error
            })
    }

    /// Add an entry and header to the CAS and EAV, respectively
    pub fn add_header_for_entry(
        &mut self,
        entry: &Entry,
        header: &ChainHeader,
    ) -> Result<(), HolochainError> {
        let eavi = EntityAttributeValueIndex::new(
            &entry.address(),
            &Attribute::EntryHeader,
            &header.address(),
        )?;
        self.add(header)?;
        self.meta_storage.write().unwrap().add_eavi(&eavi)?;
        Ok(())
    }

    pub fn mark_aspect_as_held(
        &mut self,
        entry_address: EntryHash,
        entry_aspect_address: AspectHash,
    ) {
        self.holding_map
            .entry(entry_address)
            .or_insert_with(|| HashSet::new())
            .insert(entry_aspect_address);
    }

    pub fn get_all_held_entry_addresses(&self) -> &Vec<Address> {
        &self.holding_list
    }

    pub(crate) fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> PersistenceResult<BTreeSet<EntityAttributeValueIndex>> {
        self.meta_storage.read().unwrap().fetch_eavi(query)
    }

    pub(crate) fn add_eavi(
        &mut self,
        eavi: &EntityAttributeValueIndex,
    ) -> PersistenceResult<Option<EntityAttributeValueIndex>> {
        self.meta_storage.write().unwrap().add_eavi(&eavi)
    }

    pub fn actions(&self) -> &HashMap<ActionWrapper, Result<Address, HolochainError>> {
        &self.actions
    }

    pub(crate) fn actions_mut(
        &mut self,
    ) -> &mut HashMap<ActionWrapper, Result<Address, HolochainError>> {
        &mut self.actions
    }

    pub(crate) fn next_queued_holding_workflow(
        &self,
    ) -> Option<(&PendingValidation, Option<Duration>)> {
        self.queued_holding_workflows
            .iter()
            .skip_while(|(_pending, maybe_delay)| {
                if let Some((time_of_dispatch, delay)) = maybe_delay {
                    let maybe_time_elapsed = time_of_dispatch.elapsed();
                    if let Ok(time_elapsed) = maybe_time_elapsed {
                        if time_elapsed < *delay {
                            return true;
                        }
                    }
                }
                false
            })
            .map(|(pending, maybe_delay)| {
                (
                    pending,
                    maybe_delay
                        .map(|(_time, duration)| Some(duration))
                        .unwrap_or(None),
                )
            })
            .next()
    }

    pub(crate) fn has_queued_holding_workflow(&self, pending: &PendingValidation) -> bool {
        self.queued_holding_workflows
            .iter()
            .any(|(current, _)| current == pending)
    }

    pub(crate) fn queued_holding_workflows(
        &self,
    ) -> &VecDeque<(PendingValidation, Option<(SystemTime, Duration)>)> {
        &self.queued_holding_workflows
    }
}

impl GetContent for DhtStore {
    fn get_raw(&self, address: &Address) -> HcResult<Option<Content>> {
        Ok((*self.content_storage.read().unwrap()).fetch(address)?)
    }
}

impl AddContent for DhtStore {
    fn add<T: AddressableContent>(&mut self, content: &T) -> HcResult<()> {
        (*self.content_storage.write().unwrap())
            .add(content)
            .map_err(|e| e.into())
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
        let mut store = DhtStore::new(
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
