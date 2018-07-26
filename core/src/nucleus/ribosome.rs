// In this example we execute a contract funciton exported as "_call"
#[cfg(test)]
extern crate wabt;

use holochain_wasm_utils::SinglePageAllocation;
use holochain_wasm_utils::HcApiReturnCode;

use instance::Observer;
use serde_json;
use state;
use std::sync::mpsc::Sender;

use nucleus::memory::*;

use wasmi::{
    self, Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind,
    ValueType,
};

//--------------------------------------------------------------------------------------------------
// HC API FUNCTION IMPLEMENTATIONS
//--------------------------------------------------------------------------------------------------


/// List of all the API functions available in Nucleus
#[repr(usize)]
enum HcApiFuncIndex {
    /// Print debug information in the console
    /// print(s : String)
    PRINT = 0,
    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String) -> Hash
    COMMIT,
    // Add new API function index here
    // ...
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
    let bin_arg = runtime.memory_manager.read(&allocation);

    // deserialize complex argument
    let arg = String::from_utf8(bin_arg);
    // Handle failure silently
    if let Err(_) = arg {
        return Ok(None);
    }
    let arg = arg.unwrap().to_string();
    println!("{}", arg);
    runtime.print_output.push_str(&arg);
    Ok(None)
}


/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug)]
struct CommitInputStruct {
    entry_type_name: String,
    entry_content: String,
}


/// HcApiFuncIndex::COMMIT function code
/// args: [0] encoded MemoryAllocation as u32
/// expected complex argument: r#"{"entry_type_name":"post","entry_content":"hello"}"#
/// Returns an HcApiReturnCode as I32
fn invoke_commit(runtime: &mut Runtime, args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {

    assert!(args.len() == 1);

    // Read complex argument serialized in memory
    let encoded_allocation: u32 = args.nth(0);
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation.expect("received error instead of valid encoded allocation");
    let bin_arg = runtime.memory_manager.read(&allocation);

    // deserialize complex argument
    let arg = String::from_utf8(bin_arg).unwrap();
    let res_entry: Result<CommitInputStruct, _> = serde_json::from_str(&arg);
    // Exit on error
    if res_entry.is_err() {
        // Return Error code in i32 format
        return Ok(Some(RuntimeValue::I32(
            HcApiReturnCode::ERROR_SERDE_JSON as i32,
        )));
    }

    // Create Chain Entry
    let entry_input = res_entry.unwrap();
    let entry =
        ::hash_table::entry::Entry::new(&entry_input.entry_type_name, &entry_input.entry_content);

    // Create Commit Action
    let action_commit = ::state::Action::Agent(::agent::Action::Commit(entry.clone()));

    // Send Action and block for result
    // TODO #97 - Dispatch with observer so we can check if the action did its job without errors
    ::instance::dispatch_action_and_wait(
        &runtime.action_channel,
        &runtime.observer_channel,
        action_commit.clone(),
        // TODO #131 - add timeout argument and return error on timeout
        // REDUX_DEFAULT_TIMEOUT_MS,
    );
    // TODO #97 - Return error if timeout or something failed
    // return Err(_);

    // Hash entry
    let hash_str = entry.hash();

    // Write Hash of Entry in memory in output format
    let result_str = format!("{{\"hash\":\"{}\"}}", hash_str);
    let mut result: Vec<_> = result_str.into_bytes();
    result.push(0); // Add string terminate character (important)

    let allocation_of_result = runtime.memory_manager.write(result);
    if allocation_of_result.is_err() {
        return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds))
    }

    // Return encoded allocation of result
    let encoded_allocation = allocation_of_result.unwrap().encode();
    Ok(Some(RuntimeValue::I32(encoded_allocation as i32)))
}


//--------------------------------------------------------------------------------------------------
// Wasm call
//--------------------------------------------------------------------------------------------------

/// Object holding data to pass around to invoked API functions
#[derive(Clone, Debug)]
pub struct Runtime {
    print_output: String,
    pub result: String,
    action_channel: Sender<state::ActionWrapper>,
    observer_channel: Sender<Observer>,
    memory_manager : SinglePageManager,
}


///
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

    // let mut ref_memory_manager = MemoryPageManager::new(wasm_instance.clone());

    // write input arguments for module call in memory Buffer
    let input_parameters: Vec<_> = parameters.unwrap_or_default();

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        print_output: String::new(),
        result: String::new(),
        action_channel: action_channel.clone(),
        observer_channel: observer_channel.clone(),
        // memory_manager: ref_memory_manager.clone(),
        memory_manager: SinglePageManager::new(wasm_instance.clone()),
    };

    // scope for mutable borrow of runtime
    let encoded_allocation_of_input: u32;
    {
        let mut_runtime = &mut runtime;
        let allocation_of_input = mut_runtime.memory_manager.write(input_parameters);
        encoded_allocation_of_input = allocation_of_input.unwrap().encode();
    }

    // scope for mutable borrow of runtime
    let encoded_allocation_of_output: i32;
    {
        let mut_runtime = &mut runtime;

        // invoke function in wasm instance
        // arguments are info for wasm on how to retrieve complex input arguments
        // which have been set in memory module
        encoded_allocation_of_output = wasm_instance
          .invoke_export(
              format!("{}_dispatch", function_name).as_str(),
              &[
                  RuntimeValue::I32(encoded_allocation_of_input as i32),
              ],
              mut_runtime,
          )?
          .unwrap()
          .try_into()
          .unwrap();
    }

    let allocation_of_output = SinglePageAllocation::new(encoded_allocation_of_output as u32);

    // retrieve invoked wasm function's result that got written in memory
    if let Ok(valid_allocation) = allocation_of_output {
       let result = runtime.memory_manager.read(&valid_allocation);
       runtime.result = String::from_utf8(result).unwrap();
    }

    Ok(runtime.clone())
}