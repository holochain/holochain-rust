use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_core_types::{error::HcResult, json::JsonString, signature::Signature};
use holochain_dpki::keypair::generate_random_sign_keypair;
use holochain_wasm_utils::api_serialization::crypto::CryptoArgs;
use lib3h_sodium::secbuf::SecBuf;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_crypto(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let crypto_args = match CryptoArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_sign failed to deserialize SignArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let signature = context
        .execute_conductor_crypto_api_function(crypto_args.payload.clone(),crypto_args.method)
        .map(|sig| JsonString::from_json(&sig));

    context.log(format!(
        "debug/zome: signature of data:{:?} by:{:?} is:{:?}",
        crypto_args.payload, context.agent_id, signature
    ));

    runtime.store_result(signature)
}