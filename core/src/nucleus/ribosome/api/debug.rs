use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Debug function code
/// args: [0] encoded MemoryAllocation as u64
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I64
pub fn invoke_debug(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let payload = runtime.load_json_string_from_args(args);

    runtime.context.log(format!("debug/dna: '{}'", payload));

    ribosome_success!()
}
