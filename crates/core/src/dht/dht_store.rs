use crate::{
    content_store::GetContent,
    dht::{
        aspect_map::{AspectMap, AspectMapBare},
        pending_validations::{PendingValidationWithTimeout, ValidationTimeout},
    },
};
use holochain_core_types::{
    chain_header::ChainHeader,
    crud_status::CrudStatus,
    eav::{Attribute, EaviQuery, EntityAttributeValueIndex},
    entry::Entry,
    error::{HcResult, HolochainError},
    network::entry_aspect::EntryAspect,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::{
    cas::content::{Address, AddressableContent, Content},
    eav::{EavFilter, IndexFilter},
    txn::{Cursor, CursorProviderDyn, CursorRwDyn, PersistenceManagerDyn},
};
use regex::Regex;

use crate::{dht::pending_validations::PendingValidation, state::StateWrapper};
use holochain_json_api::error::JsonResult;
use holochain_persistence_api::error::PersistenceResult;
use std::{
    collections::{BTreeSet, VecDeque},
    convert::TryFrom,
    sync::Arc,
    time::Duration,
};

/// The state-slice for the DHT.
/// Holds the CAS and EAVi that's used for the agent's local shard
/// as well as the holding list, i.e. list of all entries held for the DHT.
#[derive(Clone, Debug)]
pub struct DhtStore {
    persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>>,
    // Storages holding local shard data
    /// All the entry aspects that the network has told us to hold
    holding_map: AspectMap,

    pub(crate) queued_holding_workflows: VecDeque<PendingValidationWithTimeout>,
}

impl PartialEq for DhtStore {
    fn eq(&self, other: &DhtStore) -> bool {
        self.holding_map == other.holding_map
            && (*self.persistence_manager).get_id() == (*other.persistence_manager).get_id()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, DefaultJson)]
pub struct DhtStoreSnapshot {
    pub holding_map: AspectMapBare,
    pub queued_holding_workflows: VecDeque<PendingValidationWithTimeout>,
}

impl From<&StateWrapper> for DhtStoreSnapshot {
    fn from(state: &StateWrapper) -> Self {
        DhtStoreSnapshot {
            holding_map: state.dht().get_holding_map().bare().clone(),
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

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
        EavFilter::predicate(move |attr: Attribute| match attr {
            Attribute::LinkTag(query_link_type, query_tag)
            | Attribute::RemovedLink(query_link_type, query_tag) => {
                link_type_regex.is_match(&query_link_type) && tag_regex.is_match(&query_tag)
            }
            _ => false,
        }),
        None.into(),
        IndexFilter::LatestByAttribute,
        Some(EavFilter::single(Attribute::RemovedLink(link_type, tag))),
    ))
}
impl CursorProviderDyn<Attribute> for DhtStore {
    fn create_cursor(&self) -> PersistenceResult<Box<dyn Cursor<Attribute>>> {
        self.persistence_manager.create_cursor()
    }
    fn create_cursor_rw(&self) -> PersistenceResult<Box<dyn CursorRwDyn<Attribute>>> {
        self.persistence_manager.create_cursor_rw()
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl DhtStore {
    // LifeCycle
    // =========
    pub fn new(persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>>) -> Self {
        DhtStore {
            persistence_manager,
            holding_map: AspectMap::new(),
            queued_holding_workflows: VecDeque::new(),
        }
    }

    pub fn new_from_snapshot(
        persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>>,
        snapshot: DhtStoreSnapshot,
    ) -> Self {
        let mut new_dht_store = Self::new(persistence_manager);
        new_dht_store.holding_map = snapshot.holding_map.into();
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
        let filtered = self
            .persistence_manager
            .eav()
            .fetch_eavi(&get_links_query)?;
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
            EavFilter::predicate(move |attr: Attribute| match attr {
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
        Ok(self.persistence_manager.eav().fetch_eavi(&query)?)
    }

    /// Get all headers for an entry by first looking in the DHT meta store
    /// for header addresses, then resolving them with the DHT CAS
    pub fn get_headers(&self, entry_address: Address) -> HcResult<Vec<ChainHeader>> {
        let cursor: Box<dyn Cursor<Attribute>> = self.create_cursor()?;
        // fetch all EAV references to chain headers for this entry
        let eavis = cursor
            .fetch_eavi(&EaviQuery::new(
                Some(entry_address).into(),
                Some(Attribute::EntryHeader).into(),
                None.into(),
                IndexFilter::LatestByAttribute,
                None,
            ))
            .map_err(|e| {
                let e: HolochainError = format!("Persistence error: {:?}", e).into();
                e
            })?
            .into_iter();

        // get the header addresses
        let maybe_entries: Vec<Option<Entry>> = eavis
            .map(|eavi| eavi.value())
            // fetch the header content from CAS
            .map(|address| {
                let entry: Option<Entry> = cursor.get(&address)?;
                Ok(entry)
            })
            // rearrange
            .collect::<Result<Vec<Option<_>>, HolochainError>>()?;

        let chain_headers = maybe_entries
            .into_iter()
            // ignore None values
            .flatten()
            .map(|entry| match entry {
                Entry::ChainHeader(chain_header) => Ok(chain_header),
                _ => Err(HolochainError::ErrorGeneric(
                    "Unexpected non-chain_header entry".to_string(),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(chain_headers)
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
        let cursor: Box<dyn CursorRwDyn<Attribute>> = self.create_cursor_rw()?;
        cursor.add(header)?;
        cursor.add_eavi(&eavi)?;
        cursor.commit()?;
        Ok(())
    }

    pub fn mark_aspect_as_held(&mut self, aspect: &EntryAspect) {
        self.holding_map.add(aspect);
    }

    pub fn get_holding_map(&self) -> &AspectMap {
        &self.holding_map
    }

    pub(crate) fn fetch_eavi(
        &self,
        query: &EaviQuery,
    ) -> PersistenceResult<BTreeSet<EntityAttributeValueIndex>> {
        self.persistence_manager.eav().fetch_eavi(query)
    }

    pub(crate) fn next_queued_holding_workflow(
        &self,
    ) -> Option<(PendingValidation, Option<Duration>)> {
        self.queued_holding_workflows
            .clone()
            .into_iter()
            // filter so only free pending (those without dependencies also pending) are considered
            .filter(free_pending_filter(&self.queued_holding_workflows))
            // skip those for which the sleep delay has not elapsed
            .skip_while(|PendingValidationWithTimeout { timeout, .. }| {
                if let Some(ValidationTimeout {
                    time_of_dispatch,
                    delay,
                }) = timeout
                {
                    let maybe_time_elapsed = time_of_dispatch.elapsed();
                    if let Ok(time_elapsed) = maybe_time_elapsed {
                        if time_elapsed < *delay {
                            return true;
                        }
                    }
                }
                false
            })
            .map(|PendingValidationWithTimeout { pending, timeout }| {
                (pending, timeout.map(|t| Some(t.delay)).unwrap_or(None))
            })
            .next()
    }

    pub(crate) fn has_exact_queued_holding_workflow(&self, pending: &PendingValidation) -> bool {
        self.queued_holding_workflows.iter().any(
            |PendingValidationWithTimeout {
                 pending: current, ..
             }| current == pending,
        )
    }

    pub(crate) fn has_same_queued_holding_worfkow(&self, pending: &PendingValidation) -> bool {
        self.queued_holding_workflows.iter().any(
            |PendingValidationWithTimeout {
                 pending: current, ..
             }| {
                current.entry_with_header.header.entry_address()
                    == pending.entry_with_header.header.entry_address()
                    && current.workflow == pending.workflow
            },
        )
    }

    pub(crate) fn queued_holding_workflows(&self) -> &VecDeque<PendingValidationWithTimeout> {
        &self.queued_holding_workflows
    }
}

use im::HashSet;

fn free_pending_filter<I>(pending: &I) -> Box<dyn Fn(&PendingValidationWithTimeout) -> bool>
where
    I: IntoIterator<Item = PendingValidationWithTimeout> + Clone,
{
    // collect up the address of everything we have in the pending queue
    let unique_pending: HashSet<Address> = pending
        .clone()
        .into_iter()
        .map(|p| p.pending.entry_with_header.entry.address())
        .collect();

    Box::new(move |p| {
        p.pending
            .dependencies
            .iter()
            .all(|dep_addr| !unique_pending.contains(dep_addr))
    })
}

impl GetContent for DhtStore {
    fn get_raw(&self, address: &Address) -> HcResult<Option<Content>> {
        Ok((*self.persistence_manager.cas()).fetch(address)?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        dht::pending_validations::{PendingValidationStruct, ValidatingWorkflow},
        network::entry_with_header::EntryWithHeader,
    };
    use holochain_core_types::{
        chain_header::test_chain_header_with_sig,
        entry::{test_entry, test_entry_a, test_entry_b, test_entry_c},
    };

    #[test]
    fn get_headers_roundtrip() {
        let mut store = DhtStore::new(Arc::new(
            holochain_persistence_api::txn::new_example_persistence_manager().unwrap(),
        ));
        let entry = test_entry();
        let header1 = test_chain_header_with_sig("sig1");
        let header2 = test_chain_header_with_sig("sig2");
        store.add_header_for_entry(&entry, &header1).unwrap();
        store.add_header_for_entry(&entry, &header2).unwrap();
        let headers = store.get_headers(entry.address()).unwrap();
        assert_eq!(headers, vec![header1, header2]);
    }

    fn pending_validation_for_entry(
        entry: Entry,
        dependencies: Vec<Address>,
    ) -> PendingValidationWithTimeout {
        let header = test_chain_header_with_sig("sig1");
        let mut pending_struct = PendingValidationStruct::new(
            EntryWithHeader { entry, header },
            ValidatingWorkflow::HoldEntry,
        );
        pending_struct.dependencies = dependencies;
        PendingValidationWithTimeout::new(Arc::new(pending_struct.clone()), None)
    }

    #[test]
    fn test_dependency_resolution_no_dependencies() {
        // A and B have no dependencies. Both should be free
        let a = pending_validation_for_entry(test_entry_a(), Vec::new());
        let b = pending_validation_for_entry(test_entry_b(), Vec::new());
        let pending_list = vec![a.clone(), b.clone()];
        assert_eq!(
            pending_list
                .clone()
                .into_iter()
                .filter(free_pending_filter(&pending_list))
                .collect::<Vec<_>>(),
            vec![a, b]
        );
    }

    #[test]
    fn test_dependency_resolution_chain() {
        // A depends on B and B depends on C. C should be free
        let a = pending_validation_for_entry(test_entry_a(), vec![test_entry_b().address()]);
        let b = pending_validation_for_entry(test_entry_b(), vec![test_entry_c().address()]);
        let c = pending_validation_for_entry(test_entry_c(), vec![]);
        let pending_list = vec![a.clone(), b.clone(), c.clone()];
        assert_eq!(
            pending_list
                .clone()
                .into_iter()
                .filter(free_pending_filter(&pending_list))
                .collect::<Vec<_>>(),
            vec![c]
        );
    }

    #[test]
    fn test_dependency_resolution_tree() {
        // A depends on B and C. B and C should be free
        let a = pending_validation_for_entry(
            test_entry_a(),
            vec![test_entry_b().address(), test_entry_c().address()],
        );
        let b = pending_validation_for_entry(test_entry_b(), vec![]);
        let c = pending_validation_for_entry(test_entry_c(), vec![]);
        let pending_list = vec![a.clone(), b.clone(), c.clone()];
        assert_eq!(
            pending_list
                .clone()
                .into_iter()
                .filter(free_pending_filter(&pending_list))
                .collect::<Vec<_>>(),
            vec![b, c]
        );
    }
}
