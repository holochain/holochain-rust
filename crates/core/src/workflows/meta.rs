use crate::{NEW_RELIC_LICENSE_KEY};
use holochain_core_types::{hdk_version::HDK_VERSION, HDK_HASH};
use holochain_wasm_types::meta::{MetaArgs, MetaMethod, MetaResult};
use crate::wasm_engine::runtime::Runtime;

/// ZomeApiFunction::Meta function code
/// args: [0] encoded MemoryAllocation as u64
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_meta(runtime: &mut Runtime, meta_args: MetaArgs) -> Result<MetaResult, ()> {
    Ok(match meta_args.method {
        MetaMethod::Version => MetaResult::Version(HDK_VERSION.to_string()),
        MetaMethod::Hash => MetaResult::Hash(HDK_HASH.to_string()),
    })
}

#[cfg(test)]
#[cfg(not(windows))]
mod test_super {
    use crate::wasm_engine::{
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
