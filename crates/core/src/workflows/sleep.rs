use crate::{NEW_RELIC_LICENSE_KEY};
use std::{thread, time::Duration};
use crate::wasm_engine::runtime::Runtime;

/// ZomeApiFunction::Sleep function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_sleep(_: &mut Runtime, nanos: u64) -> Result<(), ()> {
    thread::sleep(Duration::from_nanos(nanos));
    Ok(())
}
