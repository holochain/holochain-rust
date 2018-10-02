use nucleus::ribosome::api::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use holochain_wasm_utils::error::RibosomeReturnCode;

/// ZomeApiFunction::Debug function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
pub fn invoke_debug(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let arg = runtime.load_utf8_from_args(args);

    println!("{}", arg);
    let _ = runtime.context.log(&arg);
    ribosome_return_code!(Success)
}

#[cfg(test)]
pub mod tests {
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function_runtime, ZomeApiFunction},
        Defn,
    };

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
        let (_runtime, logger) =
            test_zome_api_function_runtime(ZomeApiFunction::Debug.as_str(), test_args_bytes());
        let result = logger.lock();
        match result {
            Err(_) => assert!(false),
            Ok(logger) => {
                assert_eq!(format!("{:?}", logger.log), "[\"foo\"]".to_string());
            }
        }
    }
}
