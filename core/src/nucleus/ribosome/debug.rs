use nucleus::ribosome::{runtime_args_to_utf8, Runtime};
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// HcApiFuncIndex::DEBUG function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
pub fn invoke_debug(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let arg = runtime_args_to_utf8(runtime, args);

    // @TODO debug instead of print here (remove print entirely)
    // @see https://github.com/holochain/holochain-rust/issues/93
    println!("{}", arg);
    let _ = runtime.context.log(&arg);
    Ok(Some(RuntimeValue::I32(0 as i32)))
}

#[cfg(test)]
pub mod tests {
    use nucleus::ribosome::tests::test_zome_api_function_runtime;

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
        let (_runtime, logger) = test_zome_api_function_runtime("debug", test_args_bytes());
        let result = logger.lock();
        match result {
            Err(_) => assert!(false),
            Ok(logger) => {
                assert_eq!(format!("{:?}", logger.log), "[\"foo\"]".to_string());
            }
        }
    }
}
