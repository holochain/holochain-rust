use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use wasmi::RuntimeArgs;

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = dbg!(runtime.load_json_string_from_args(&args));

    let signature = runtime.context()?.sign(args_str.into());

    runtime.store_result(signature)
}
