use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use holochain_core_types::{hdk_version::HDK_VERSION, GIT_HASH};
use holochain_wasm_utils::api_serialization::meta::{MetaArgs, MetaMethod, MetaResult};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Meta function code
/// args: [0] encoded MemoryAllocation as u64
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I64
pub fn invoke_meta(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;

    let args_str = runtime.load_json_string_from_args(&args);
    let meta_args = match MetaArgs::try_from(args_str.clone()) {
        Ok(args) => args,
        // Exit on error
        Err(error) => {
            log_error!(
                context,
                "zome: invoke_meta failed to \
                 deserialize arguments: {:?} with error {:?}",
                args_str,
                error
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let method = match meta_args.method {
        MetaMethod::Version => MetaResult::Version(HDK_VERSION.to_string()),
        MetaMethod::Hash => MetaResult::Hash(GIT_HASH.to_string()),
    };

    let result = Ok(method);

    runtime.store_result(result)
}

#[cfg(test)]
#[cfg(not(windows))]
mod test_super {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::hdk_version::HDK_VERSION;
    use holochain_json_api::json::*;
    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_meta_functions() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::Meta.as_str(),
            format!(r#"{{ "method" : "Version" }}"#).as_bytes().to_vec(),
        );
        let call_result_json = format!(
            r#"{{"ok":true,"value":"{{\"Version\":\"{}\"}}","error":"null"}}"#,
            HDK_VERSION.to_string()
        );
        assert_eq!(JsonString::from_json(&*call_result_json), call_result,);
    }
}
