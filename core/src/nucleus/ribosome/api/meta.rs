use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use holochain_wasm_utils::api_serialization::meta::{MetaArgs,MetaMethod,MetaResult};
use wasmi::{RuntimeArgs, RuntimeValue};
use holochain_core_types::GIT_HASH;
use holochain_core_types::hdk_version::HDK_VERSION;
use std::convert::TryFrom;

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
            log_error!(context,
                "zome: invoke_emit_signal failed to \
                 deserialize arguments: {:?} with error {:?}",
                args_str, error
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let method = match meta_args.method
    {
        MetaMethod::Version => MetaResult::Version(HDK_VERSION.to_string()),
        MetaMethod::Hash => MetaResult::Hash(GIT_HASH.to_string())
    };

    let result = Ok(method);

   runtime.store_result(result)
}