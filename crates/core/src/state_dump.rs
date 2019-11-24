use crate::{
    action::QueryKey,
    context::Context,
    dht::{aspect_map::AspectMapBare, pending_validations::PendingValidation},
    network::direct_message::DirectMessage,
    nucleus::{ZomeFnCall, ZomeFnCallState},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError};
use holochain_json_api::json::JsonString;
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{
    collections::VecDeque,
    convert::TryInto,
    sync::Arc,
    time::{Duration, SystemTime},
};

#[derive(Serialize)]
pub struct StateDump {
    pub queued_calls: Vec<ZomeFnCall>,
    pub running_calls: Vec<(ZomeFnCall, Option<ZomeFnCallState>)>,
    pub call_results: Vec<(ZomeFnCall, Result<JsonString, HolochainError>)>,
    pub query_flows: Vec<QueryKey>,
    pub validation_package_flows: Vec<Address>,
    pub direct_message_flows: Vec<(String, DirectMessage)>,
    pub queued_holding_workflows: VecDeque<(PendingValidation, Option<(SystemTime, Duration)>)>,
    pub held_aspects: AspectMapBare,
    pub source_chain: Vec<ChainHeader>,
}

impl From<Arc<Context>> for StateDump {
    fn from(context: Arc<Context>) -> StateDump {
        let (agent, nucleus, network, dht) = {
            let state_lock = context.state().expect("No state?!");
            (
                (*state_lock.agent()).clone(),
                (*state_lock.nucleus()).clone(),
                (*state_lock.network()).clone(),
                (*state_lock.dht()).clone(),
            )
        };

        let source_chain: Vec<ChainHeader> = agent.iter_chain().collect();
        let source_chain: Vec<ChainHeader> = source_chain.into_iter().rev().collect();

        let queued_calls: Vec<ZomeFnCall> = nucleus.queued_zome_calls.into_iter().collect();
        let invocations = nucleus.hdk_function_calls;
        let running_calls: Vec<(ZomeFnCall, Option<ZomeFnCallState>)> = nucleus
            .running_zome_calls
            .into_iter()
            .map(|call| {
                let state = invocations.get(&call).cloned();
                (call, state)
            })
            .collect();
        let call_results: Vec<(ZomeFnCall, Result<_, _>)> =
            nucleus.zome_call_results.into_iter().collect();

        let query_flows: Vec<QueryKey> = network
            .get_query_results
            //using iter so that we don't copy this again and again if it is a scheduled job that runs everytime
            //it might be slow if copied
            .iter()
            .filter(|(_, result)| result.is_none())
            .map(|(key, _)| key.clone())
            .collect();

        let validation_package_flows: Vec<Address> = network
            .get_validation_package_results
            .into_iter()
            .filter(|(_, result)| result.is_none())
            .map(|(address, _)| address)
            .collect();

        let direct_message_flows: Vec<(String, DirectMessage)> = network
            .direct_message_connections
            .into_iter()
            .map(|(s, dm)| (s.clone(), dm.clone()))
            .collect();

        let queued_holding_workflows = dht.queued_holding_workflows().clone();

        let held_aspects = dht.get_holding_map().bare().clone();

        StateDump {
            queued_calls,
            running_calls,
            call_results,
            query_flows,
            validation_package_flows,
            direct_message_flows,
            queued_holding_workflows,
            held_aspects,
            source_chain,
        }
    }
}

pub fn address_to_content_and_type(
    address: &Address,
    context: Arc<Context>,
) -> Result<(String, String), HolochainError> {
    let raw_content = context
        .dht_storage
        .read()?
        .fetch(address)?
        .ok_or(HolochainError::EntryNotFoundLocally)?;
    let maybe_entry: Result<Entry, _> = raw_content.clone().try_into();
    if let Ok(entry) = maybe_entry {
        let mut entry_type = entry.entry_type().to_string();
        let content = match entry {
            Entry::Dna(_) => String::from("DNA omitted"),
            Entry::AgentId(agent_id) => agent_id.nick,
            Entry::LinkAdd(link) | Entry::LinkRemove((link, _)) => format!(
                "({}#{})\n\t{} => {}",
                link.link.link_type(),
                link.link.tag(),
                link.link.base(),
                link.link.target(),
            ),
            Entry::App(app_type, app_value) => {
                entry_type = app_type.to_string();
                app_value.to_string()
            }
            _ => entry.content().to_string(),
        };
        Ok((entry_type, content))
    } else {
        Ok((String::from("UNKNOWN"), raw_content.to_string()))
    }
}
