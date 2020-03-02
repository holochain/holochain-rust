use crate::{
    action::ActionWrapper,
    agent::{
        chain_store::ChainStore,
        state::{AgentState, AgentStateSnapshot},
    },
    content_store::GetContent,
    context::Context,
    dht::dht_store::DhtStore,
    network::state::NetworkState,
    nucleus::state::{NucleusState, NucleusStateSnapshot},
    
};
use holochain_conductor_lib_api::ConductorApi;
use holochain_core_types::{
    chain_header::ChainHeader,
    dna::Dna,
    eav::{Attribute, EaviQuery},
    entry::{entry_type::EntryType, Entry},
    error::{HcResult, HolochainError},
};
use holochain_locksmith::RwLock;
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::IndexFilter,
};

use crate::dht::dht_store::DhtStoreSnapshot;
use std::{convert::TryInto, sync::Arc, time::SystemTime};

pub const ACTION_PRUNE_MS: u64 = 60000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionResponse<T> {
    pub created_at: SystemTime,
    pub response: T,
}

impl<T> ActionResponse<T> {
    pub fn new(response: T) -> Self {
        ActionResponse::<T> {
            created_at: SystemTime::now(),
            response,
        }
    }

    pub fn response(&self) -> &T {
        &self.response
    }
}

/// The Store of the Holochain instance Object, according to Redux pattern.
/// It's composed of all sub-module's state slices.
/// To plug in a new module, its state slice needs to be added here.
#[autotrace]
#[derive(Clone, PartialEq, Debug)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    dht: Arc<DhtStore>,
    network: Arc<NetworkState>,
    pub conductor_api: ConductorApi,
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl State {
    pub fn new(context: Arc<Context>) -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246

        let chain_cas = &(*context).chain_storage;
        let dht_cas = &(*context).dht_storage;
        let eav = context.eav_storage.clone();
        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(
                ChainStore::new(chain_cas.clone()),
                context.agent_id.address(),
            )),
            dht: Arc::new(DhtStore::new(dht_cas.clone(), eav)),
            network: Arc::new(NetworkState::new()),
            conductor_api: context.conductor_api.clone(),
        }
    }

    pub fn new_with_agent(context: Arc<Context>, agent_state: AgentState) -> Self {
        Self::new_with_agent_and_nucleus(context, agent_state, NucleusState::new())
    }

    pub fn new_with_agent_and_nucleus(
        context: Arc<Context>,
        agent_state: AgentState,
        nucleus_state: NucleusState,
    ) -> Self {
        let cas = context.dht_storage.clone();
        let eav = context.eav_storage.clone();

        let dht_store = DhtStore::new(cas.clone(), eav.clone());
        Self::new_with_agent_nucleus_dht(context, agent_state, nucleus_state, dht_store)
    }

    pub fn new_with_agent_nucleus_dht(
        context: Arc<Context>,
        agent_state: AgentState,
        mut nucleus_state: NucleusState,
        dht_store: DhtStore,
    ) -> Self {
        let cas = context.dht_storage.clone();
        //let eav = context.eav_storage.clone();

        nucleus_state.dna = Self::get_dna(&agent_state, cas.clone()).ok();

        State {
            nucleus: Arc::new(nucleus_state),
            agent: Arc::new(agent_state),
            dht: Arc::new(dht_store),
            network: Arc::new(NetworkState::new()),
            conductor_api: context.conductor_api.clone(),
        }
    }

    fn get_dna(
        agent_state: &AgentState,
        cas: Arc<RwLock<dyn ContentAddressableStorage>>,
    ) -> HcResult<Dna> {
        let dna_entry_header = agent_state
            .chain_store()
            .iter_type(&agent_state.top_chain_header(), &EntryType::Dna)
            .last()
            .ok_or_else(|| {
                HolochainError::ErrorGeneric(
                    "No DNA entry found in source chain while creating state from agent"
                        .to_string(),
                )
            })?;
        let json = (*cas.read().unwrap()).fetch(dna_entry_header.entry_address())?;
        let entry: Entry = json.map(|e| e.try_into()).ok_or_else(|| {
            HolochainError::ErrorGeneric(
                "No DNA entry found in storage while creating state from agent".to_string(),
            )
        })??;
        match entry {
            Entry::Dna(dna) => Ok(*dna),
            _ => Err(HolochainError::SerializationError(
                "Tried to get Dna from non-Dna Entry".into(),
            )),
        }
    }

    #[autotrace]
    pub fn reduce(&self, action_wrapper: ActionWrapper) -> Self {
        let _span_guard = ht::push_span_with(|span| {
            span.child_("reduce-inner", |s| {
                s.tag(ht::Tag::new(
                    "action_wrapper",
                    format!("{:?}", action_wrapper),
                ))
                .start()
            })
            .into()
        });
        State {
            nucleus: crate::nucleus::reduce(Arc::clone(&self.nucleus), &self, &action_wrapper),
            agent: crate::agent::state::reduce(Arc::clone(&self.agent), &self, &action_wrapper),
            dht: crate::dht::dht_reducers::reduce(Arc::clone(&self.dht), &action_wrapper),
            network: crate::network::reducers::reduce(
                Arc::clone(&self.network),
                &self,
                &action_wrapper,
            ),
            conductor_api: self.conductor_api.clone(),
        }
    }

    pub fn nucleus(&self) -> Arc<NucleusState> {
        Arc::clone(&self.nucleus)
    }

    pub fn agent(&self) -> Arc<AgentState> {
        Arc::clone(&self.agent)
    }

    pub fn dht(&self) -> Arc<DhtStore> {
        Arc::clone(&self.dht)
    }

    pub fn network(&self) -> Arc<NetworkState> {
        Arc::clone(&self.network)
    }

    pub fn try_from_snapshots(
        context: Arc<Context>,
        agent_snapshot: AgentStateSnapshot,
        nucleus_snapshot: NucleusStateSnapshot,
        dht_store_snapshot: DhtStoreSnapshot,
    ) -> HcResult<State> {
        let agent_state = AgentState::new_with_top_chain_header(
            ChainStore::new(context.chain_storage.clone()),
            agent_snapshot.top_chain_header().map(|h| h.to_owned()),
            context.agent_id.address(),
        );
        let nucleus_state = NucleusState::from(nucleus_snapshot);
        let dht_store = DhtStore::new_from_snapshot(
            context.dht_storage.clone(),
            context.eav_storage.clone(),
            dht_store_snapshot,
        );
        Ok(State::new_with_agent_nucleus_dht(
            context,
            agent_state,
            nucleus_state,
            dht_store,
        ))
    }

    /// Get all headers for an entry by first looking in the DHT meta store
    /// for header addresses, then resolving them with the DHT CAS
    pub fn get_headers(&self, entry_address: Address) -> Result<Vec<ChainHeader>, HolochainError> {
        let headers: Vec<ChainHeader> = self
            .agent()
            .iter_chain()
            .filter(|h| h.entry_address() == &entry_address)
            .collect();
        let header_addresses: Vec<Address> = headers.iter().map(|h| h.address()).collect();
        let mut dht_headers = self
            .dht()
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
            // don't include the chain header twice
            .filter(|a| !header_addresses.contains(a))
            // fetch the header content from CAS
            .map(|address| self.dht().get(&address))
            // rearrange
            .collect::<Result<Vec<Option<_>>, _>>()
            .map(|r| {
                r.into_iter()
                    // ignore None values
                    .flatten()
                    .map(|entry| match entry {
                        Entry::ChainHeader(chain_header) => Ok(chain_header),
                        _ => Err(HolochainError::ErrorGeneric(
                            "Non chain-header entry found".to_string(),
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()
            })??;
        {
            let mut all_headers = headers;
            all_headers.append(&mut dht_headers);
            Ok(all_headers)
        }
    }
}

/// This type wraps (decorates) InnerState with an option and re-exports and delegates all
/// methods of InnerState.
/// It owns the InnerState and keeps it in a Option so that it can be dropped explicitly.
/// It also adds a function `drop(&mut self)` which sets the option to None which will
/// drop the InnerState.
#[derive(Clone, PartialEq, Debug)]
pub struct StateWrapper {
    state: Option<State>,
}

#[autotrace]
impl StateWrapper {
    pub fn drop_inner_state(&mut self) {
        self.state = None;
    }

    pub fn new(context: Arc<Context>) -> Self {
        StateWrapper {
            state: Some(State::new(context)),
        }
    }

    pub fn new_with_agent(context: Arc<Context>, agent_state: AgentState) -> Self {
        StateWrapper {
            state: Some(State::new_with_agent(context, agent_state)),
        }
    }

    pub fn new_with_agent_and_nucleus(
        context: Arc<Context>,
        agent_state: AgentState,
        nucleus_state: NucleusState,
    ) -> Self {
        StateWrapper {
            state: Some(State::new_with_agent_and_nucleus(
                context,
                agent_state,
                nucleus_state,
            )),
        }
    }

    #[autotrace]
    pub fn reduce(&self, action_wrapper: ActionWrapper) -> Self {
        StateWrapper {
            state: Some(
                self.state
                    .as_ref()
                    .expect("Tried to use dropped state")
                    .reduce(action_wrapper),
            ),
        }
    }

    pub fn nucleus(&self) -> Arc<NucleusState> {
        Arc::clone(
            &self
                .state
                .as_ref()
                .expect("Tried to use dropped state")
                .nucleus,
        )
    }

    pub fn agent(&self) -> Arc<AgentState> {
        Arc::clone(
            &self
                .state
                .as_ref()
                .expect("Tried to use dropped state")
                .agent,
        )
    }

    pub fn dht(&self) -> Arc<DhtStore> {
        Arc::clone(&self.state.as_ref().expect("Tried to use dropped state").dht)
    }

    pub fn network(&self) -> Arc<NetworkState> {
        Arc::clone(
            &self
                .state
                .as_ref()
                .expect("Tried to use dropped state")
                .network,
        )
    }

    pub fn get_headers(&self, entry_address: Address) -> Result<Vec<ChainHeader>, HolochainError> {
        self.state
            .as_ref()
            .expect("Tried to use dropped state")
            .get_headers(entry_address)
    }

    pub fn conductor_api(&self) -> ConductorApi {
        self.state
            .as_ref()
            .expect("Tried to use dropped state")
            .conductor_api
            .clone()
    }
}

impl From<State> for StateWrapper {
    fn from(state: State) -> StateWrapper {
        StateWrapper { state: Some(state) }
    }
}

pub fn test_store(context: Arc<Context>) -> State {
    State::new(context)
}
