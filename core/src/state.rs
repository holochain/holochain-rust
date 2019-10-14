use crate::{
    action::ActionWrapper,
    agent::{
        chain_store::ChainStore,
        state::{AgentState, AgentStateSnapshot},
    },
    context::Context,
    dht::dht_store::DhtStore,
    network::state::NetworkState,
    nucleus::state::{NucleusState, NucleusStateSnapshot},
};
use holochain_core_types::{
    chain_header::ChainHeader,
    dna::Dna,
    eav::{Attribute, EaviQuery},
    entry::{entry_type::EntryType, Entry},
    error::{HcResult, HolochainError},
    sync::{HcRwLock as RwLock},
};
use holochain_conductor_api_api::ConductorApi;
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::IndexFilter,
};

use crate::dht::dht_store::DhtStoreSnapshot;
use std::{
    collections::HashSet,
    convert::TryInto,
    sync::{Arc},
};

/// The Store of the Holochain instance Object, according to Redux pattern.
/// It's composed of all sub-module's state slices.
/// To plug in a new module, its state slice needs to be added here.
#[derive(Clone, PartialEq, Debug)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    dht: Arc<DhtStore>,
    network: Arc<NetworkState>,
    // @TODO eventually drop stale history
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub history: HashSet<ActionWrapper>,
    pub conductor_api: ConductorApi,
}

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
            history: HashSet::new(),
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
            history: HashSet::new(),
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
            .ok_or_else(|| HolochainError::ErrorGeneric(
                "No DNA entry found in source chain while creating state from agent".to_string(),
            ))?;
        let json = (*cas.read().unwrap()).fetch(dna_entry_header.entry_address())?;
        let entry: Entry = json
            .map(|e| e.try_into())
            .ok_or_else(|| HolochainError::ErrorGeneric(
                "No DNA entry found in storage while creating state from agent".to_string(),
            ))??;
        match entry {
            Entry::Dna(dna) => Ok(*dna),
            _ => Err(HolochainError::SerializationError(
                "Tried to get Dna from non-Dna Entry".into(),
            )),
        }
    }

    pub fn reduce(&self, action_wrapper: ActionWrapper) -> Self {
        let mut new_state = State {
            nucleus: crate::nucleus::reduce(Arc::clone(&self.nucleus), &self, &action_wrapper),
            agent: crate::agent::state::reduce(Arc::clone(&self.agent), &self, &action_wrapper),
            dht: crate::dht::dht_reducers::reduce(Arc::clone(&self.dht), &action_wrapper),
            network: crate::network::reducers::reduce(
                Arc::clone(&self.network),
                &self,
                &action_wrapper,
            ),
            history: self.history.clone(),
            conductor_api: self.conductor_api.clone(),
        };

        new_state.history.insert(action_wrapper);
        new_state
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
        let dht_store = DhtStore::new_with_holding_list(
            context.dht_storage.clone(),
            context.eav_storage.clone(),
            dht_store_snapshot.holding_list,
        );
        Ok(State::new_with_agent_nucleus_dht(
            context.clone(),
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
            .meta_storage()
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
            // don't include the chain header twice
            .filter(|a| !header_addresses.contains(a))
            // fetch the header content from CAS
            .map(|a| self.dht().content_storage().read().unwrap().fetch(&a))
            // rearrange
            .collect::<Result<Vec<Option<_>>, _>>()
            .map(|r| {
                r.into_iter()
                    // ignore None values
                    .flatten()
                    .map(|content| ChainHeader::try_from_content(&content))
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

    pub fn history(&self) -> HashSet<ActionWrapper> {
        self.state
            .as_ref()
            .expect("Tried to use dropped state")
            .history
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
