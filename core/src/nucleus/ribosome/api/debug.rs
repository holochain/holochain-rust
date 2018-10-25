use holochain_core_types::error::RibosomeReturnCode;
use nucleus::ribosome::api::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::Debug function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
pub fn invoke_debug(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let args_str = runtime.load_utf8_from_args(args);

    println!("{}", args_str);

    Ok(Some(RuntimeValue::I32(i32::from(
        RibosomeReturnCode::Success,
    ))))
}

#[cfg(test)]
pub mod tests {
    use holochain_core_types::{error::RibosomeReturnCode, json::JsonString};
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeApiFunction},
        Defn,
    };
    use std::convert::TryFrom;

    /// dummy string for testing print zome API function
    pub fn test_debug_string() -> String {
        "foo".to_string()
    }

    /// dummy bytes for testing print based on test_print_string()
    pub fn test_args_bytes() -> Vec<u8> {
        test_debug_string().into_bytes()
    }

    #[test]
    /// test that bytes passed to debug end up in the log
    fn test_debug() {
        let (runtime, logger) =
            test_zome_api_function_runtime(ZomeApiFunction::Debug.as_str(), test_args_bytes());
        let logger = logger.lock().unwrap();
        assert_eq!(
            RibosomeReturnCode::Success,
            RibosomeReturnCode::try_from(runtime.result)
                .expect("could not deserialize RibosomeReturnCode"),
        );
        assert_eq!(
            JsonString::from(format!("{:?}", logger.log)),
            JsonString::from(
                "[\"Zome Function did not allocate memory: \\\'test\\\' return code: Success\"]",
            ),
        );
    }
}
