use crate::{
    wasm_engine::{api::ZomeApiResult},
    NEW_RELIC_LICENSE_KEY,
};
use std::{thread, time::Duration};
use std::sync::Arc;
use crate::context::Context;

/// ZomeApiFunction::Sleep function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_sleep(context: Arc<Context>, nanos: u64) -> ZomeApiResult {
    thread::sleep(Duration::from_nanos(nanos));
    Ok(())
}
