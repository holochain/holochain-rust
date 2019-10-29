use crate::trace::{tracer, LogContext};

// result of no-op is no-op
pub fn handle_drop_entry_result(log_context: &LogContext) {
    tracer(&log_context, "handle_drop_entry_result");
}
