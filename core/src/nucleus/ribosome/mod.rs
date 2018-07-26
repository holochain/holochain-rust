// In this example we execute a contract funciton exported as "_call"
mod commit;
mod print;
mod get;

use instance::Observer;
use state;
use std::sync::mpsc::Sender;
use nucleus::ribosome::print::invoke_print;
use nucleus::ribosome::commit::invoke_commit;
use nucleus::ribosome::get::invoke_get;

use wasmi::{
    self, Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryRef,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

//--------------------------------------------------------------------------------------------------
// HC API FUNCTION IMPLEMENTATIONS
//--------------------------------------------------------------------------------------------------

/// Enumeration of all possible return codes that an HC API function can return
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum HcApiReturnCode {
    SUCCESS = 0,
    ERROR_SERDE_JSON,
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
    /// Get an entry from source chain by key (header hash)
    /// get(key: String) -> Pair
    GET,
    // Add new API function index here
    // ...
}

//--------------------------------------------------------------------------------------------------
// Wasm call
//--------------------------------------------------------------------------------------------------

pub const RESULT_OFFSET: u32 = 0;

/// Object holding data to pass around to invoked API functions
#[derive(Clone, Debug)]
pub struct Runtime {
    print_output: Vec<u32>,
    pub result: String,
    action_channel: Sender<state::ActionWrapper>,
    observer_channel: Sender<Observer>,
    memory: MemoryRef,
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
                index if index == HcApiFuncIndex::PRINT as usize => invoke_print(self, &args),
                index if index == HcApiFuncIndex::COMMIT as usize => invoke_commit(self, &args),
                index if index == HcApiFuncIndex::GET as usize => invoke_get(self, &args),
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
                    Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                    HcApiFuncIndex::COMMIT as usize,
                ),
                "get" => FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                    HcApiFuncIndex::GET as usize,
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
    wasm_memory
        .set(RESULT_OFFSET, &params)
        .expect("memory should be writable");

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        print_output: vec![],
        result: String::new(),
        action_channel: action_channel.clone(),
        observer_channel: observer_channel.clone(),
        memory: wasm_memory.clone(),
    };

    // invoke function in wasm instance
    // arguments are info for wasm on how to retrieve complex input arguments
    // which have been set in memory module
    let i32_result_length: i32 = wasm_instance
        .invoke_export(
            format!("{}_dispatch", function_name).as_str(),
            &[
                RuntimeValue::I32(RESULT_OFFSET as i32),
                RuntimeValue::I32(params.len() as i32),
            ],
            &mut runtime,
        )?
        .unwrap()
        .try_into()
        .unwrap();

    // retrieve invoked wasm function's result that got written in memory
    let result = wasm_memory
        .get(RESULT_OFFSET, i32_result_length as usize)
        .expect("Successfully retrieve the result");
    runtime.result = String::from_utf8(result).unwrap();

    Ok(runtime.clone())
}
