use crate::{action::ActionWrapper, network::actions::ActionResponse};
use holochain_core_types::{cas::content::Address, entry::EntryWithMeta, error::HolochainError};
use holochain_net::p2p_network::P2pNetwork;
use snowflake;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type ActionMap = HashMap<ActionWrapper, ActionResponse>;
type GetEntryWithMetaResult = Option<Result<Option<EntryWithMeta>, HolochainError>>;

#[derive(Clone, Debug)]
pub struct NetworkState {
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub actions: ActionMap,
    pub network: Option<Arc<Mutex<P2pNetwork>>>,
    pub dna_hash: Option<String>,
    pub agent_id: Option<String>,
    pub get_entry_with_meta_results: HashMap<Address, GetEntryWithMetaResult>,
    id: snowflake::ProcessUniqueId,
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &NetworkState) -> bool {
        self.id == other.id
    }
}

impl NetworkState {
    pub fn new() -> Self {
        NetworkState {
            actions: HashMap::new(),
            network: None,
            dna_hash: None,
            agent_id: None,
            get_entry_with_meta_results: HashMap::new(),
            id: snowflake::ProcessUniqueId::new(),
        }
    }

    pub fn actions(&self) -> ActionMap {
        self.actions.clone()
    }
}
