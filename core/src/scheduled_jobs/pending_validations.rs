use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    nucleus::actions::remove_pending_validation::remove_pending_validation,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    error::error::HolochainError,
    json::JsonString,
};
use std::{fmt, sync::Arc, thread};

pub type PendingValidation = Arc<PendingValidationStruct>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, DefaultJson)]
pub enum ValidatingWorkflow {
    HoldEntry,
    HoldLink,
}

impl fmt::Display for ValidatingWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidatingWorkflow::HoldEntry => write!(f, "HoldEntryWorkflow"),
            ValidatingWorkflow::HoldLink => write!(f, "HoldLinkWorkflow"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DefaultJson)]
pub struct PendingValidationStruct {
    pub entry_with_header: EntryWithHeader,
    pub dependencies: Vec<Address>,
    pub workflow: ValidatingWorkflow,
}

fn retry_validation(pending: PendingValidation, context: Arc<Context>) {
    thread::spawn(move || {
        let result = match pending.workflow {
            ValidatingWorkflow::HoldLink => {
                context.block_on(hold_link_workflow(&pending.entry_with_header, &context))
            }
            ValidatingWorkflow::HoldEntry => context.block_on(hold_entry_workflow(
                &pending.entry_with_header,
                context.clone(),
            )),
        };

        if Err(HolochainError::ValidationPending) != result {
            remove_pending_validation(
                pending.entry_with_header.entry.address(),
                pending.workflow.clone(),
                &context,
            );
        }
    });
}

pub fn run_pending_validations(context: Arc<Context>) {
    let pending_validations = context
        .state()
        .unwrap()
        .nucleus()
        .pending_validations
        .clone();

    pending_validations.iter().for_each(|(_, pending)| {
        context.log(format!(
            "debug/scheduled_jobs/run_pending_validations: found pending validation for {}: {}",
            pending.entry_with_header.entry.entry_type(),
            pending.entry_with_header.entry.address()
        ));
        retry_validation(pending.clone(), context.clone());
    });
}
