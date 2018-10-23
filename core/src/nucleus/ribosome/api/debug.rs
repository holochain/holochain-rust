use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::Debug function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
pub fn invoke_debug(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let payload = runtime.load_utf8_from_args(args);
    println!("{}", payload);
    // TODO #502 - log in logger as DEBUG log-level
    runtime
        .context
        .log(&format!("zome_log:DEBUG: '{}'", payload))
        .expect("Logger should work");

    // Return Ribosome Success Code
    Ok(Some(RuntimeValue::I32(0 as i32)))
}

#[cfg(test)]
pub mod tests {
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
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

    /// test that bytes passed to debug end up in the log
    #[test]
    fn test_zome_api_function_debug() {
        let (call_result, context) =
            test_zome_api_function(ZomeApiFunction::Debug.as_str(), test_args_bytes());
        assert!(call_result.is_empty());
        assert_eq!(
            "[\"zome_log:DEBUG: \\\'foo\\\'\", \"Zome Function \\\'test\\\' returned: Success\"]",
            format!("{}", (*context.logger.lock().unwrap()).dump()),
        );
    }
}
