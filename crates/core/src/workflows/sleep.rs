use std::{thread, time::Duration};
use std::sync::Arc;
use crate::context::Context;
use crate::workflows::InfallibleWorkflowResult;

/// ZomeApiFunction::Sleep function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn sleep_workflow(_: Arc<Context>, nanos: &u64) -> InfallibleWorkflowResult {
    thread::sleep(Duration::from_nanos(*nanos));
    Ok(())
}
