use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_core_types::signature::Provenance;
use holochain_sodium::secbuf::SecBuf;
use holochain_wasm_utils::api_serialization::verify_signature::VerifySignatureArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::VerifySignature function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_verify_signature(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let verification_args = match VerifySignatureArgs::try_from(args_str.clone()) {
        Ok(verify_signature_input) => verify_signature_input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_verify_signature failed to deserialize SerializedEntry: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    context.log(format!(
        "debug/zome: using provenance:{:?} to verify data:{:?}",
        verification_args.provenance.clone(),
        verification_args.payload.clone()
    ));

    let Provenance(addr, signature) = verification_args.provenance;

    let mut message_data = SecBuf::with_insecure_from_string(verification_args.payload.clone());
    let mut signature_data = SecBuf::with_insecure_from_string(signature.into());

    let verification_result =
        holochain_dpki::utils::verify(addr.into(), &mut message_data, &mut signature_data);

    runtime.store_result(verification_result)
}

/// test that bytes passed to debug end up in the log
#[test]
fn test_zome_api_function_verify() {
    let (call_result, context) = test_zome_api_function(
        ZomeApiFunction::Sign.as_str(),
        r#"{ "payload": "this is data" }"#.as_bytes().to_vec(),
    );
    assert_eq!(JsonString::from(r#"{"ok":true,"value":"xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA==","error":"null"}"#), call_result,);

    let args = format!(r#"{{ "provenance": ["{}","xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA=="], "payload": "this is data" }}"#,context.agent_id.address());
    let (call_result, _) = test_zome_api_function(
        ZomeApiFunction::VerifySignature.as_str(),
        args.as_bytes().to_vec(),
    );

    assert_eq!(
        JsonString::from(r#"{"ok":true,"value":"true","error":"null"}"#),
        call_result,
    );
}
