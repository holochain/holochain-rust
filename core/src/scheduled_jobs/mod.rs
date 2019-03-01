pub mod pending_validations;

use crate::context::Context;
use std::sync::Arc;

pub fn create_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        context.log("debug/scheduled_jobs: tick");
        pending_validations::run_pending_validations(context.clone());
    }
}
