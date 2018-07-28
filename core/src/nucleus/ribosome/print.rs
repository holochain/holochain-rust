use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// HcApiFuncIndex::PRINT function code
pub fn invoke_print(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    let arg: u32 = args.nth(0);
    runtime.print_output.push(arg);
    Ok(None)
}

/// HcApiFuncIndex::PRINT function code
/// args: [0] encoded MemoryAllocation as u32
/// Expecting a string as complex input argument
/// Returns an HcApiReturnCode as I32
fn invoke_print(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    assert!(args.len() == 1);

    // Read complex argument serialized in memory
    let encoded_allocation: u32 = args.nth(0);
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation.expect("received error instead of valid encoded allocation");
    let bin_arg = runtime.memory_manager.read(allocation);

    // deserialize complex argument
    let arg = String::from_utf8(bin_arg);
    // Handle failure silently
    if arg.is_err() {
        return Ok(None);
    }
    let arg = arg.unwrap().to_string();
    println!("{}", arg);
    runtime.print_output.push_str(&arg);
    Ok(None)
}

#[cfg(test)]
mod tests {
    extern crate wabt;

    use self::wabt::Wat2Wasm;
    use instance::Observer;
    use nucleus::ribosome::call;
    use std::sync::mpsc::channel;

    fn test_wasm() -> Vec<u8> {
        let wasm_binary = Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                r#"
                (module
                    (type (;0;) (func (result i32)))
                    (type (;1;) (func (param i32)))
                    (type (;2;) (func))
                    (import "env" "print" (func $print (type 1)))
                    (func (export "test_print_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                        i32.const 1337
                        call $print
                        i32.const 0)
                    (func $rust_eh_personality (type 2))
                    (table (;0;) 1 1 anyfunc)
                    (memory (;0;) 17)
                    (global (;0;) (mut i32) (i32.const 1049600))
                    (export "memory" (memory 0))
                    (export "rust_eh_personality" (func $rust_eh_personality)))
            "#,
            )
            .unwrap();

        wasm_binary.as_ref().to_vec()
    }

    #[test]
    fn test_print() {
        let (action_channel, _) = channel::<::state::ActionWrapper>();
        let (tx_observer, _observer) = channel::<Observer>();
        let runtime = call(
            &action_channel,
            &tx_observer,
            test_wasm(),
            "test_print",
            None,
        ).expect("test_print should be callable");
        assert_eq!(runtime.print_output.len(), 1);
        assert_eq!(runtime.print_output[0], 1337)
    }
}
