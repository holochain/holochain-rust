use instance::Observer;
use state;
use std::sync::mpsc::Sender;
use serde_json;

use wasmi::{
    self, Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
    MemoryRef,
};

/// Object to hold VM data that we want out of the VM
#[derive(Clone, Debug)]
pub struct Runtime {
    print_output: Vec<u32>,
    pub result: String,
    action_channel: Sender<state::ActionWrapper>,
    observer_channel: Sender<Observer>,
    memory : MemoryRef,
}

/// List of all the API functions available in Nucleus
#[repr(usize)]
enum HcApiFuncIndex {
    /// Print debug information in the console
    /// print(...)
    PRINT = 0,
    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String) -> Hash
    COMMIT,
    // Add new API function index here
    // ...
}

pub const RESULT_OFFSET: u32 = 0;

/// HcApiFuncIndex::PRINT function code
fn invoke_print(runtime : & mut Runtime, args: RuntimeArgs)
    -> Result<Option<RuntimeValue>, Trap>
{
    let arg: u32 = args.nth(0);
    runtime.print_output.push(arg);
    Ok(None)
}


#[derive(Deserialize, Default, Debug)]
struct CommitInputStruct {
    entry_type_name: String,
    entry_content: String,
}


/// HcApiFuncIndex::COMMIT function code
/// fn commit(data: *mut c_char, params_len: usize) -> *mut c_char;
/// args: [0] len of complex arguments in memory
/// todo add offset argument?
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
fn invoke_commit(runtime : & mut Runtime, args: RuntimeArgs)
  -> Result<Option<RuntimeValue>, Trap>
{
    // println!(" --- invoke_commit START");
    // println!("\t args = {:?}", args);

    // Read complex argument serialized in memory
    let arg_len: u32 = args.nth(0);
    let bin_arg =
        runtime.memory
          .get(RESULT_OFFSET, arg_len as usize)
          .expect("Successfully retrieve the arguments");

    // deserialize argument
    let arg = String::from_utf8(bin_arg).unwrap();
    // println!("\t arg = {}", arg);
    let res_entry : Result<CommitInputStruct, _> = serde_json::from_str(&arg);
    // Exit on error
    if let Err(_) = res_entry {
        // FIXME write error in memory
        return Ok(Some(RuntimeValue::I32(42)));
    }

    // Create Chain Entry
    let entry_input = res_entry.unwrap();
    let entry = ::chain::entry::Entry::new(
        &entry_input.entry_type_name,
        &entry_input.entry_content,
    );
    // println!("\t entry = {:?}", entry);

    // Create Commit Action
    let action_commit = ::state::Action::Agent(::agent::Action::Commit(entry.clone()));

    // Send Action and block for result
    ::instance::dispatch_action_and_wait(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_commit.clone(),
        // TODO - add timeout argument and return error on timeout
        //2000, // FIXME have global const for default timeout
    );
    // TODO - return error on timeout
    // return Err(_);

    // Hash entry
    let hash_str = entry.hash();

    // Write Hash of Entry in memory in output format
    let params_str = format!("{{\"hash\":\"{}\"}}", hash_str);
    // println!(" --- params_str = {}", params_str);
    let mut params: Vec<_> = params_str.into_bytes();
    // let mut params: Vec<_> = "{\"hash\":\"QmXyZ\"}".to_string().into_bytes();
    params.push(0);
    // println!(" --- params = {:?}", params);
    runtime.memory.set(0, &params).expect("memory should be writable");

    // Return success in i32 format
    // println!(" --- invoke_commit STOP");
    Ok(Some(RuntimeValue::I32(0)))
}


/// Executes an exposed function in a wasm binary
pub fn call(
    action_channel: &Sender<state::ActionWrapper>,
    observer_channel: &Sender<Observer>,
    wasm: Vec<u8>,
    function_name: &str,
    parameters: Option<Vec<u8>>,
) -> Result<Runtime, InterpreterError> {

    // Create wasm module from wasm binary
    let module = wasmi::Module::from_buffer(wasm).unwrap();

    // Describe invokable functions form within Zome
    impl Externals for Runtime {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            match index {
                index if index == HcApiFuncIndex::PRINT as usize => { invoke_print(self, args) }
                index if index == HcApiFuncIndex::COMMIT as usize => { invoke_commit(self, args) }
                // Add API function code here
                // ....
                _ => panic!("unknown function index"),
            }
        }
    }

    // Define invokable functions form within Zome
    struct RuntimeModuleImportResolver;
    impl ModuleImportResolver for RuntimeModuleImportResolver {
        fn resolve_func(
            &self,
            field_name: &str,
            _signature: &Signature,
        ) -> Result<FuncRef, InterpreterError> {
            let func_ref = match field_name {
                "print" => FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], None),
                    HcApiFuncIndex::PRINT as usize,
                ),
                "commit" => FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                    HcApiFuncIndex::COMMIT as usize,
                ),
                // Add API function here
                // ....
                _ => {
                    return Err(InterpreterError::Function(format!(
                        "host module doesn't export function with name {}",
                        field_name
                    )))
                }
            };
            Ok(func_ref)
        }
    }

    // Create Imports with previously described Resolver
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    // Create module instance from wasm module, and without starting it
    let wasm_instance = ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .assert_no_start();


    // get wasm memory reference from module
    let wasm_memory = wasm_instance
        .export_by_name("memory")
        .expect("all modules compiled with rustc should have an export named 'memory'; qed")
        .as_memory()
        .expect("in module generated by rustc export named 'memory' should be a memory; qed")
        .clone();

    // write arguments for module call at beginning of memory module
    let params: Vec<_> = parameters.unwrap_or_default();
    wasm_memory.set(0, &params).expect("memory should be writable");

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        print_output: vec![],
        result: String::new(),
        action_channel: action_channel.clone(),
        observer_channel: observer_channel.clone(),
        memory : wasm_memory.clone(),
    };

    // invoke function in wasm instance
    // arguments are info for wasm on how to retrieve complex input arguments
    // which have been set in memory module
    let i32_result_length: i32 =
        wasm_instance
        .invoke_export(
            format!("{}_dispatch", function_name).as_str(),
            &[RuntimeValue::I32(0), RuntimeValue::I32(params.len() as i32)],
            &mut runtime, // external state for data passing
        )?
        .unwrap()
        .try_into()
        .unwrap();

    // retrieve invoked wasm function's result that got written in memory
    let result =
        wasm_memory
        .get(RESULT_OFFSET, i32_result_length as usize)
        .expect("Successfully retrieve the result");
    runtime.result = String::from_utf8(result).unwrap();

    Ok(runtime.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use wabt::Wat2Wasm;

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
