pub mod pending_validations;
pub mod state_dump;

use crate::context::Context;
use std::sync::Arc;

pub fn create_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        context.log("debug/scheduled_jobs: tick");
        state_dump::state_dump(context.clone());
        pending_validations::run_pending_validations(context.clone());
    }
}
