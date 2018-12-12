use crate::{
    network::{actions::custom_send::custom_send, direct_message::CustomDirectMessage},
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use futures::executor::block_on;
use holochain_wasm_utils::api_serialization::send::SendArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Send function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: SendArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_send(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let args = match SendArgs::try_from(args_str) {
        Ok(input) => input,
        Err(..) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    let message = CustomDirectMessage {
        payload: Ok(args.payload),
        zome: runtime.zome_call.zome_name.clone(),
    };

    let result = block_on(custom_send(args.to_agent, message, &runtime.context));

    runtime.store_result(result)
}
