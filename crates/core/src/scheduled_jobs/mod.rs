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
use std::sync::Arc;

pub fn create_state_dump_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        //log_debug!(context, "scheduled_jobs: tick");
        if context.state_dump_logging {
            state_dump::state_dump(context.clone());
        }
    }
}

pub fn create_validation_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        log_debug!(context, "Checking holding queue...");
        loop {
            let maybe_holding_workflow = context
                .state()
                .expect("Couldn't get state in run_pending_validations")
                .dht()
                .next_queued_holding_workflow();
            if let Some(pending) = maybe_holding_workflow {
                log_debug!(context, "Found queued validation: {:?}", pending);
                pop_next_holding_workflow(pending.clone(), context.clone());

                let result = pending_validations::run_holding_workflow(&pending, context.clone());

                match result {
                    // If we couldn't run the validation due to unresolved dependencies,
                    // we have to re-add this entry at the end of the queue:
                    Err(HolochainError::ValidationPending) => {
                        queue_holding_workflow(pending, context.clone())
                    }
                    Err(e) => log_error!(
                        context,
                        "Error running holding workflow for {:?}: {:?}",
                        pending,
                        e,
                    ),
                    Ok(()) => log_info!(context, "Successfully processed: {:?}", pending),
                }
            } else {
                break;
            }
        }
    }
}
