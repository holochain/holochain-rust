use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_json_api::json::*;
use holochain_wasm_utils::api_serialization::crypto::CryptoArgs;
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

    let message = context
        .conductor_api
        .execute(crypto_args.payload.clone(), crypto_args.method.clone())
        .map(|sig| JsonString::from_json(&sig));

    context.log(format!(
        "debug/zome: crypto method {:?} of data:{:?} by:{:?} is:{:?}",
        crypto_args.method, crypto_args.payload, context.agent_id, message
    ));

    runtime.store_result(message)
}

#[cfg(test)]
mod test_super {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::json::JsonString;
    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_encrypt() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            r#"{ "payload": "this is data", "method" : "Encrypt" }"#
                .as_bytes()
                .to_vec(),
        );
        assert_eq!(
            JsonString::from_json(
                r#"{"ok":true,"value":"FJ/KKN5d7VHUu+8jKiMWuDtIBZclOBETQ8Gnkw==","error":"null"}"#
            ),
            call_result,
        );
    }

    #[test]
    fn test_zome_api_function_decrypt() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Crypto.as_str(),
            r#"{ "payload": "FJ/KKN5d7VHUu+8jKiMWuDtIBZclOBETQ8Gnkw==", "method" : "Decrypt" }"#
                .as_bytes()
                .to_vec(),
        );
        assert_eq!(
            JsonString::from_json(r#"{"ok":true,"value":"this is data","error":"null"}"#),
            call_result,
        );
    }

}
