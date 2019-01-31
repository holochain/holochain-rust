use crate::nucleus::ribosome::{api::ZomeApiResult, Runtime};
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::Debug function code
/// args: [0] encoded MemoryAllocation as u64
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I64
pub fn invoke_debug(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    let payload = runtime.load_json_string_from_args(args);

    zome_call_data
        .context
        .log(format!("debug/dna: '{}'", payload));

    ribosome_success!()
}

#[cfg(test)]
// tests broken because debug is too spammy
// @see https://github.com/holochain/holochain-rust/issues/928
#[cfg(feature = "broken-tests")]
pub mod tests {
    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::json::JsonString;

    /// dummy string for testing print zome API function
    pub fn test_debug_string() -> String {
        "foo".to_string()
    }

    /// dummy bytes for testing print based on test_print_string()
    pub fn test_args_bytes() -> Vec<u8> {
        test_debug_string().into_bytes()
    }

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_debug() {
        let (call_result, context) =
            test_zome_api_function(ZomeApiFunction::Debug.as_str(), test_args_bytes());
        println!(
            "test_zome_api_function_debug call_result: {:?}",
            call_result
        );
        assert_eq!(JsonString::null(), call_result,);
        let expected_in_log =
       "\"debug/dna: \\\'foo\\\'\", \"debug/zome: Zome Function \\\'test\\\' returned: Success\"";
        let log_contents = format!("{}", (*context.logger.lock().unwrap()).dump());
        assert!(log_contents.contains(expected_in_log));
    }
}
