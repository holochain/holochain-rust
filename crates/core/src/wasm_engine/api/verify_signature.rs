use crate::{
    wasm_engine::{api::ZomeApiResult, Runtime},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_dpki::utils::Verify;
use holochain_wasm_utils::api_serialization::verify_signature::VerifySignatureArgs;

/// ZomeApiFunction::VerifySignature function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_verify_signature(
    runtime: &mut Runtime,
    verification_args: VerifySignatureArgs,
) -> ZomeApiResult {
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_verify_signature(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let verification_args = match VerifySignatureArgs::try_from(args_str.clone()) {
        Ok(verify_signature_input) => verify_signature_input,
        // Exit on error
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_verify_signature failed to deserialize SerializedEntry: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    log_debug!(
        runtime.context()?,
        "zome: using provenance:{:?} to verify data:{:?}",
        verification_args.provenance.clone(),
        verification_args.payload.clone()
    );

    let verification_result = verification_args
        .provenance
        .verify(verification_args.payload.clone());

    runtime.store_result(verification_result)
}

#[cfg(test)]
mod test_super {
    use crate::{
        holochain_wasm_utils::holochain_persistence_api::cas::content::AddressableContent,
        wasm_engine::{
            api::{tests::test_zome_api_function, ZomeApiFunction},
            Defn,
        },
    };
    use holochain_json_api::json::JsonString;

    #[test]
    fn test_zome_api_function_verify() {
        let (call_result, context) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            r#"{ "payload": "this is data", "method":"Sign" }"#.as_bytes().to_vec(),
        );
        assert_eq!(
            JsonString::from_json(
                r#"{"ok":true,"value":"xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA==","error":"null"}"#
            ),
            call_result,
        );

        let args = format!(
            r#"{{ "provenance": ["{}","xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA=="], "payload": "this is data" }}"#,
            context.agent_id.address()
        );
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::VerifySignature.as_str(),
            args.as_bytes().to_vec(),
        );

        assert_eq!(
            JsonString::from_json(r#"{"ok":true,"value":"true","error":"null"}"#),
            call_result,
        );
    }
}
