pub mod state_dump;

use crate::context::Context;
use std::sync::Arc;

pub fn create_state_dump_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        //log_debug!(context, "scheduled_jobs: tick");
        if context.state_dump_logging {
            state_dump::state_dump(context.clone());
        }
    }
}
