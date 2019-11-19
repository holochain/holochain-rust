use crate::{
    context::Context,
    network::chain_pair::ChainPair,
    nucleus::actions::remove_pending_validation::remove_pending_validation,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use holochain_core_types::error::HolochainError;

use crate::workflows::{
    hold_entry_remove::hold_remove_workflow, hold_entry_update::hold_update_workflow,
    remove_link::remove_link_workflow,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use snowflake::ProcessUniqueId;
use std::{convert::TryFrom, fmt, sync::Arc, thread};

pub type PendingValidation = Arc<PendingValidationStruct>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, DefaultJson)]
pub enum ValidatingWorkflow {
    HoldEntry,
    HoldLink,
    RemoveLink,
    UpdateEntry,
    RemoveEntry,
}

impl Into<String> for ValidatingWorkflow {
    fn into(self) -> String {
        match self {
            ValidatingWorkflow::HoldEntry => String::from("HoldEntry"),
            ValidatingWorkflow::HoldLink => String::from("HoldLink"),
            ValidatingWorkflow::RemoveLink => String::from("RemoveLink"),
            ValidatingWorkflow::UpdateEntry => String::from("UpdateEntry"),
            ValidatingWorkflow::RemoveEntry => String::from("RemoveEntry"),
        }
    }
}

impl TryFrom<String> for ValidatingWorkflow {
    type Error = HolochainError;
    fn try_from(s: String) -> Result<ValidatingWorkflow, HolochainError> {
        match s.as_ref() {
            "HoldEntry" => Ok(ValidatingWorkflow::HoldEntry),
            "HoldLink" => Ok(ValidatingWorkflow::HoldLink),
            "RemoveLink" => Ok(ValidatingWorkflow::RemoveLink),
            "UpdateEntry" => Ok(ValidatingWorkflow::UpdateEntry),
            "RemoveEntry" => Ok(ValidatingWorkflow::RemoveEntry),
            _ => Err(HolochainError::SerializationError(String::from(
                "No ValidatingWorkflow",
            ))),
        }
    }
}

impl fmt::Display for ValidatingWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidatingWorkflow::HoldEntry => write!(f, "HoldEntryWorkflow"),
            ValidatingWorkflow::HoldLink => write!(f, "HoldLinkWorkflow"),
            ValidatingWorkflow::RemoveLink => write!(f, "RemoveLinkWorkflow"),
            ValidatingWorkflow::UpdateEntry => write!(f, "UpdateEntryWorkflow"),
            ValidatingWorkflow::RemoveEntry => write!(f, "RemoveEntryWorkflow"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DefaultJson)]
pub struct PendingValidationStruct {
    pub chain_pair: ChainPair,
    pub dependencies: Vec<Address>,
    pub workflow: ValidatingWorkflow,
}

fn retry_validation(pending: PendingValidation, context: Arc<Context>) {
    thread::Builder::new()
        .name(format!(
            "retry_validation/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let result = match pending.workflow {
                ValidatingWorkflow::HoldLink => {
                    context.block_on(hold_link_workflow(&pending.chain_pair, context.clone()))
                }
                ValidatingWorkflow::HoldEntry => {
                    context.block_on(hold_entry_workflow(&pending.chain_pair, context.clone()))
                }
                ValidatingWorkflow::RemoveLink => {
                    context.block_on(remove_link_workflow(&pending.chain_pair, context.clone()))
                }
                ValidatingWorkflow::UpdateEntry => {
                    context.block_on(hold_update_workflow(&pending.chain_pair, context.clone()))
                }
                ValidatingWorkflow::RemoveEntry => {
                    context.block_on(hold_remove_workflow(&pending.chain_pair, context.clone()))
                }
            };
            if Err(HolochainError::ValidationPending) != result {
                remove_pending_validation(
                    pending.chain_pair.entry().address(),
                    pending.workflow.clone(),
                    &context,
                );
            }
        })
        .expect("Could not spawn thread for retry_validation");
}

pub fn run_pending_validations(context: Arc<Context>) {
    let pending_validations = context
        .state()
        .expect("Couldn't get state in run_pending_validations")
        .nucleus()
        .pending_validations
        .clone();

    pending_validations.iter().for_each(|(_, pending)| {
        log_debug!(
            context,
            "scheduled_jobs/run_pending_validations: found pending validation for {}: {}",
            pending.chain_pair.entry().entry_type(),
            pending.chain_pair.entry().address()
        );
        retry_validation(pending.clone(), context.clone());
    });
}
