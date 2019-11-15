use crate::{
    action::QueryKey, context::Context, network::direct_message::DirectMessage,
    nucleus::ZomeFnCall, scheduled_jobs::pending_validations::ValidatingWorkflow,
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry, error::HolochainError,flamerwrapper::FlamerWrapper};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{convert::TryInto, sync::Arc};

#[derive(Serialize)]
pub struct PendingValidationDump {
    pub address: Address,
    pub dependencies: Vec<Address>,
    pub workflow: ValidatingWorkflow,
}

#[derive(Serialize)]
pub struct StateDump {
    pub running_calls: Vec<ZomeFnCall>,
    pub query_flows: Vec<QueryKey>,
    pub validation_package_flows: Vec<Address>,
    pub direct_message_flows: Vec<(String, DirectMessage)>,
    pub pending_validations: Vec<PendingValidationDump>,
    pub held_entries: Vec<Address>,
    pub source_chain: Vec<ChainHeader>,
}

impl From<Arc<Context>> for StateDump {
    fn from(context: Arc<Context>) -> StateDump {
        FlamerWrapper::start("statedump_from_context");
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

        let running_calls: Vec<ZomeFnCall> = nucleus
            .zome_calls
            .into_iter()
            .filter(|(_, result)| result.is_none())
            .map(|(call, _)| call)
            .collect();

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

        let pending_validations = nucleus
            .pending_validations
            .into_iter()
            .map(
                |(pending_validation_key, pending_validation)| PendingValidationDump {
                    address: pending_validation_key.address,
                    workflow: pending_validation_key.workflow,
                    dependencies: pending_validation.dependencies.clone(),
                },
            )
            .collect::<Vec<PendingValidationDump>>();

        let held_entries = dht.get_all_held_entry_addresses().clone();
        FlamerWrapper::end("statedump_from_context");
        StateDump {
            running_calls,
            query_flows,
            validation_package_flows,
            direct_message_flows,
            pending_validations,
            held_entries,
            source_chain,
        }
    }
}

pub fn address_to_content_and_type(
    address: &Address,
    context: Arc<Context>,
) -> Result<(String, String), HolochainError> {
    FlamerWrapper::start("address_to_content_and_type");
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
        FlamerWrapper::end("address_to_content_and_type");
        Ok((entry_type, content))
    } else {
        FlamerWrapper::end("address_to_content_and_type");
        Ok((String::from("UNKNOWN"), raw_content.to_string()))
    }
}
