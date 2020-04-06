use crate::{
    action::{ActionWrapper, QueryKey, ValidationKey},
    network::{actions::Response, direct_message::DirectMessage, query::NetworkQueryResult},
};
use boolinator::*;
use holochain_core_types::{error::HolochainError, validation::ValidationPackage};
use holochain_net::p2p_network::P2pNetwork;
use holochain_persistence_api::cas::content::Address;
use im::HashMap;
use std::time::{Duration, SystemTime};

type Actions = HashMap<ActionWrapper, Response>;

/// This represents the state of a get_validation_package network process:
/// None: process started, but no response yet from the network
/// Some(Err(_)): there was a problem at some point
/// Some(Ok(None)): no error but also no validation package -> we seem to have asked the wrong
///   agent which actually should not happen. Something weird is going on.
/// Some(Ok(Some(entry))): we have it
type GetValidationPackageResult = Option<Result<Option<ValidationPackage>, HolochainError>>;

type GetResults = Option<Result<NetworkQueryResult, HolochainError>>;

#[derive(Clone, Debug)]
pub struct NetworkState {
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    pub actions: Actions,
    pub network: Option<P2pNetwork>,
    pub dna_address: Option<Address>,
    pub agent_id: Option<String>,

    // Here are the results of every get action
    pub get_query_results: HashMap<QueryKey, GetResults>,
    pub query_timeouts: HashMap<QueryKey, (SystemTime, Duration)>,

    /// Here we store the results of get validation package processes.
    /// None means that we are still waiting for a result from the network.
    pub get_validation_package_results: HashMap<ValidationKey, GetValidationPackageResult>,
    pub get_validation_package_timeouts: HashMap<ValidationKey, (SystemTime, Duration)>,

    /// This stores every open (= waiting for response) node-to-node messages.
    /// Entries get removed when we receive an answer through Action::ResolveDirectConnection.
    pub direct_message_connections: HashMap<String, DirectMessage>,
    pub direct_message_timeouts: HashMap<String, (SystemTime, Duration)>,

    pub custom_direct_message_replys: HashMap<String, Result<String, HolochainError>>,

    id: String,
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &NetworkState) -> bool {
        self.id == other.id
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl NetworkState {
    pub fn new() -> Self {
        NetworkState {
            actions: HashMap::new(),
            network: None,
            dna_address: None,
            agent_id: None,
            get_query_results: HashMap::new(),
            query_timeouts: HashMap::new(),
            get_validation_package_results: HashMap::new(),
            get_validation_package_timeouts: HashMap::new(),
            direct_message_connections: HashMap::new(),
            direct_message_timeouts: HashMap::new(),
            custom_direct_message_replys: HashMap::new(),

            id: nanoid::simple(),
        }
    }

    pub fn actions(&self) -> Actions {
        self.actions.clone()
    }

    pub fn initialized(&self) -> Result<(), HolochainError> {
        (self.network.is_some() && self.dna_address.is_some() && self.agent_id.is_some()).ok_or(
            HolochainError::ErrorGeneric("Network not initialized".to_string()),
        )
    }
}
