use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use nucleus::ribosome::runtime_args_to_utf8;

/// HcApiFuncIndex::PRINT function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
pub fn invoke_print(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    let arg = runtime_args_to_utf8(runtime, args);

    println!("{}", arg);
    runtime.print_output.push_str(&arg);
    Ok(None)
}

#[cfg(test)]
mod tests {
    use nucleus::ribosome::tests::test_zome_api_function_runtime;

    pub fn test_print_string() -> String {
        "foo".to_string()
    }

    pub fn test_args_bytes() -> Vec<u8> {
        test_print_string().into_bytes()
    }

    #[test]
    fn test_print() {
        let runtime = test_zome_api_function_runtime("print", test_args_bytes());

        assert_eq!(
            runtime.print_output,
            test_print_string(),
        );

        // assert_eq!(runtime.print_output.len(), 1);
        // assert_eq!(runtime.print_output[0], test_args_bytes())
    }
}
