use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let _args_str = dbg!(runtime.load_json_string_from_args(&args));

    // TODO:

    ribosome_success!()
}
