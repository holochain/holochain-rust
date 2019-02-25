pub mod pending_validations;

use crate::context::Context;
use std::sync::Arc;

pub fn create_callback(context: Arc<Context>) -> impl 'static + FnMut() + Sync + Send {
    move || {
        pending_validations::run_pending_validations(context.clone());
    }
}