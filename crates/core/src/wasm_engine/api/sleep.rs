use crate::wasm_engine::{api::ZomeApiResult, Runtime};
use std::{thread, time::Duration};

/// ZomeApiFunction::Sleep function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sleep(_runtime: &Runtime, nanos: u64) -> ZomeApiResult {
    thread::sleep(Duration::from_nanos(nanos));

    ribosome_success!()
}
