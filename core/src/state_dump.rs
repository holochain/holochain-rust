use crate::nucleus::ZomeFnCall;
use crate::action::QueryKey;
use holochain_persistence_api::cas::content::Address;
use crate::network::direct_message::DirectMessage;
use crate::scheduled_jobs::pending_validations::ValidatingWorkflow;
use crate::context::Context;
use std::sync::Arc;

#[derive(Serialize)]
pub struct PendingValidationDump{
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
}

impl From<Arc<Context>> for StateDump {
    fn from(context: Arc<Context>) -> StateDump {
        let (nucleus, network, dht) = {
            let state_lock = context.state().expect("No state?!");
            (
                (*state_lock.nucleus()).clone(),
                (*state_lock.network()).clone(),
                (*state_lock.dht()).clone(),
            )
        };

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
            .filter(|(_,result)|result.is_none())
            .map(|(key,_)|key.clone())
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
            .map(|(pending_validation_key, pending_validation)| {
                PendingValidationDump {
                    address: pending_validation_key.address,
                    workflow: pending_validation_key.workflow,
                    dependencies: pending_validation.dependencies.clone(),
                }
            })
            .collect::<Vec<PendingValidationDump>>();

        let held_entries = dht.get_all_held_entry_addresses().clone();

        StateDump {
            running_calls, query_flows, validation_package_flows, direct_message_flows,
            pending_validations, held_entries
        }
    }
}