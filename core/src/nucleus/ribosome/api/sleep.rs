use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use std::convert::TryFrom;
use std::{
    time::Duration, thread,
};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Sleep function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sleep(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let nanos = match u64::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    thread::sleep(Duration::from_nanos(nanos));

    ribosome_success!()
}
