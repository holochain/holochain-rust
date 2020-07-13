use crate::{
    action::{ActionWrapper, QueryKey, ValidationKey},
    network::{actions::Response, direct_message::DirectMessage, query::NetworkQueryResult},
};
use boolinator::*;
use holochain_core_types::{
    chain_header::ChainHeader,
    entry::Entry,
    error::HolochainError,
    validation::{ValidationPackage, ValidationPackageDefinition},
};
use holochain_json_api::{error::JsonError, json::JsonString};
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

/// Cached source chain data for validation
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, DefaultJson)]
pub struct ValidationCacheData {
    pub entries: Option<Vec<Entry>>,
    pub headers: Vec<ChainHeader>,
    pub cached_at: SystemTime,
}

impl ValidationCacheData {
    fn latest_header(&self) -> ChainHeader {
        // we should never add an empty headers array to the cache
        self.headers.first().unwrap().clone()
    }
}

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
    pub validation_cache: HashMap<Address, ValidationCacheData>,

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
            validation_cache: HashMap::new(),

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

    pub(crate) fn cache_validation(&mut self, agent: Address, validation: &ValidationPackage) {
        if validation.source_chain_headers.is_none() {
            return;
        }
        if self.validation_cache.contains_key(&agent) {
            let mut cache_entry = self.validation_cache.get_mut(&agent).unwrap();
            if validation.chain_header.timestamp() > cache_entry.latest_header().timestamp() {
                debug!("Vcache: updating {}:{:?}", &agent, validation.chain_header);
                debug!(
                    "header count: {}",
                    validation.source_chain_headers.as_ref().unwrap().len()
                );
                cache_entry.headers = validation.source_chain_headers.clone().unwrap();
                if validation.source_chain_entries.is_some() {
                    debug!(
                        "entry count: {}",
                        validation.source_chain_entries.as_ref().unwrap().len()
                    );
                    cache_entry.entries = validation.source_chain_entries.clone();
                }
            } else {
                debug!("Vcache: already {}:{:?}", &agent, validation.chain_header);
            }
        } else {
            debug!(
                "Vcache: initial insert {}:{:?} {:#?}",
                &agent, validation.chain_header, &validation
            );
            self.validation_cache.insert(
                agent,
                ValidationCacheData {
                    entries: validation.source_chain_entries.clone(),
                    headers: validation.source_chain_headers.as_ref().unwrap().clone(),
                    cached_at: SystemTime::now(),
                },
            );
        }
    }

    // tries to build a validation package of the given type from the cache for a
    pub fn get_validation_package_from_cache(
        &self,
        agent: Address,
        definition: &ValidationPackageDefinition,
        header: &ChainHeader,
    ) -> Option<ValidationPackage> {
        let cache_entry = self.validation_cache.get(&agent)?;

        // check all the cases where we know we can't calculate the validation package and
        // return, so the rest of the cases we can just unwrap.
        match definition {
            ValidationPackageDefinition::Entry
            | ValidationPackageDefinition::ChainEntries
            | ValidationPackageDefinition::Custom(_) => return None,
            ValidationPackageDefinition::ChainHeaders => {
                //                if cache_entry.headers.is_none() {
                return None;
                //                };
            }
            ValidationPackageDefinition::ChainFull => {
                if cache_entry.headers.len() == 0 {
                    return None;
                };
                if cache_entry.entries.is_none() {
                    return None;
                };
            }
        }

        // if we are looking for a header that was after the latest cached
        // we can't build it
        if header.timestamp() >= cache_entry.latest_header().timestamp() {
            return None;
        };

        match definition {
            ValidationPackageDefinition::ChainHeaders => Some(ValidationPackage {
                chain_header: header.clone(),
                source_chain_headers: Some(cache_entry.headers.clone()),
                source_chain_entries: None,
                custom: None,
            }),
            ValidationPackageDefinition::ChainFull => Some(ValidationPackage {
                chain_header: header.clone(),
                source_chain_headers: Some(cache_entry.headers.clone()),
                source_chain_entries: cache_entry.entries.clone(),
                custom: None,
            }),
            _ => None,
        }
    }
}
