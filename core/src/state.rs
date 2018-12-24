use crate::{
    action::ActionWrapper,
    agent::{
        chain_store::ChainStore,
        state::{AgentState, AgentStateSnapshot},
    },
    context::{ContextOnly, ContextStateful},
    dht::dht_store::DhtStore,
    network::state::NetworkState,
    nucleus::state::NucleusState,
};
use holochain_core_types::{
    cas::storage::ContentAddressableStorage,
    dna::{wasm::DnaWasm, Dna},
    entry::{entry_type::EntryType, Entry},
    error::{HcResult, HolochainError},
};
use std::{
    collections::HashSet,
    convert::TryInto,
    sync::{Arc, RwLock},
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
}

impl State {
    pub fn new(context_only: &ContextOnly) -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246

        let chain_cas = context_only.chain_storage();
        let dht_cas = context_only.dht_storage();
        let eav = context_only.eav_storage();
        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new(ChainStore::new(chain_cas.clone()))),
            dht: Arc::new(DhtStore::new(dht_cas.clone(), eav)),
            network: Arc::new(NetworkState::new()),
            history: HashSet::new(),
        }
    }

    pub fn new_with_agent(context_only: &ContextOnly, agent_state: Arc<AgentState>) -> Self {
        // @TODO file table
        // @see https://github.com/holochain/holochain-rust/pull/246

        let cas = context_only.dht_storage();
        let eav = context_only.eav_storage();

        fn get_dna(
            agent_state: &Arc<AgentState>,
            cas: Arc<RwLock<dyn ContentAddressableStorage>>,
        ) -> HcResult<Dna> {
            let dna_entry_header = agent_state
                .chain()
                .iter_type(&agent_state.top_chain_header(), &EntryType::Dna)
                .last()
                .ok_or(HolochainError::ErrorGeneric(
                    "No DNA entry found in source chain while creating state from agent"
                        .to_string(),
                ))?;
            let json = (*cas.read().unwrap()).fetch(dna_entry_header.entry_address())?;
            let entry: Entry =
                json.map(|e| e.try_into())
                    .ok_or(HolochainError::ErrorGeneric(
                        "No DNA entry found in storage while creating state from agent".to_string(),
                    ))??;
            match entry {
                Entry::Dna(dna) => Ok(dna),
                _ => Err(HolochainError::SerializationError(
                    "Tried to get Dna from non-Dna Entry".into(),
                )),
            }
        }

        let mut nucleus_state = NucleusState::new();
        nucleus_state.dna = get_dna(&agent_state, cas.clone()).ok();
        State {
            nucleus: Arc::new(nucleus_state),
            agent: agent_state,
            dht: Arc::new(DhtStore::new(cas.clone(), eav.clone())),
            network: Arc::new(NetworkState::new()),
            history: HashSet::new(),
        }
    }

    pub fn reduce(&self, context: Arc<ContextStateful>, action_wrapper: ActionWrapper) -> Self {
        let mut new_state = State {
            nucleus: crate::nucleus::reduce(
                Arc::clone(&context),
                Arc::clone(&self.nucleus),
                &action_wrapper,
            ),
            agent: crate::agent::state::reduce(
                Arc::clone(&context),
                Arc::clone(&self.agent),
                &action_wrapper,
            ),
            dht: crate::dht::dht_reducers::reduce(
                Arc::clone(&context),
                Arc::clone(&self.dht),
                &action_wrapper,
            ),
            network: crate::network::reducers::reduce(
                Arc::clone(&context),
                Arc::clone(&self.network),
                &action_wrapper,
            ),
            history: self.history.clone(),
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

    pub fn try_from_agent_snapshot(
        context: &ContextOnly,
        snapshot: AgentStateSnapshot,
    ) -> HcResult<State> {
        let agent_state = AgentState::new_with_top_chain_header(
            ChainStore::new(context.dht_storage().clone()),
            snapshot.top_chain_header().clone(),
        );
        Ok(State::new_with_agent(context, Arc::new(agent_state)))
    }

    pub fn get_dna(&self) -> Option<Dna> {
        use std::{thread, time::Duration};
        // In the case of genesis we encounter race conditions with regards to setting the DNA.
        // Genesis gets called asynchronously right after dispatching an action that sets the DNA in
        // the state, which can result in this code being executed first.
        // But we can't run anything if there is no DNA which holds the WASM, so we have to wait here.
        // TODO: use a future here
        let mut dna = None;
        let mut done = false;
        let mut tries = 0;
        while !done {
            {
                dna = self.nucleus().dna();
            }
            match dna {
                Some(_) => done = true,
                None => {
                    if tries > 10 {
                        done = true;
                    } else {
                        thread::sleep(Duration::from_millis(10));
                        tries += 1;
                    }
                }
            }
        }
        dna
    }

    pub fn get_wasm(&self, zome: &str) -> Option<DnaWasm> {
        let dna = self.get_dna().expect("Callback called without DNA set!");
        dna.get_wasm_from_zome_name(zome)
            .and_then(|wasm| Some(wasm.clone()).filter(|_| !wasm.code.is_empty()))
    }
}

pub fn test_store(context: Arc<ContextOnly>) -> State {
    State::new(&*context)
}
