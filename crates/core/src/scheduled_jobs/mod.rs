pub mod pending_validations;
pub mod state_dump;

use crate::{
    context::Context,
    dht::actions::{
        pop_next_holding_workflow::pop_next_holding_workflow,
        queue_holding_workflow::queue_holding_workflow,
    },
};
use holochain_core_types::error::HolochainError;
use holochain_persistence_api::cas::content::AddressableContent;
use std::sync::Arc;

pub fn create_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        //log_debug!(context, "scheduled_jobs: tick");
        if context.state_dump_logging {
            state_dump::state_dump(context.clone());
        }
    }
}

pub fn create_validation_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        if let Some(pending) = context
            .state()
            .expect("Couldn't get state in run_pending_validations")
            .dht()
            .next_queued_holding_workflow()
        {
            pop_next_holding_workflow(context.clone());

            log_debug!(
                context,
                "scheduled_jobs/run_validations: found queued validation for {}: {}",
                pending.entry_with_header.entry.entry_type(),
                pending.entry_with_header.entry.address()
            );

            let result = pending_validations::run_holding_workflow(pending, context.clone());

            // If we couldn't run the validation due to unresolved dependencies,
            // we have to re-add this entry at the end of the queue:
            if Err(HolochainError::ValidationPending) == result {
                queue_holding_workflow(pending.clone(), context.clone());
            }
        }
    }
}
