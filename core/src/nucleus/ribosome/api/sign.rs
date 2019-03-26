use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_wasm_utils::api_serialization::sign::SignArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Sign function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected argument: u64
/// Returns an HcApiReturnCode as I64
pub fn invoke_sign(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let sign_args = match SignArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_sign failed to deserialize SerializedEntry: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let signature = context.sign(sign_args.payload.clone());

    context.log(format!(
        "debug/zome: signature of data:{:?} by:{:?} is:{:?}",
        sign_args.payload, context.agent_id, signature
    ));

    runtime.store_result(signature)
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
    fn test_zome_api_function_sign() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Sign.as_str(),
            r#"{ "payload": "this is data" }"#.as_bytes().to_vec(),
        );
        assert_eq!(JsonString::from(r#"{"ok":true,"value":"xoEEoLF1yWM4VBNtjEwrfM/iVzjuAxxbkOyBWi0LV0+1CAH/PCs9MErnbmFeZRtQNtw7+SmVrm7Irac4lZsaDA==","error":"null"}"#), call_result,);
    }
}
