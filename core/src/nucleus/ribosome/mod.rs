mod api;
pub mod lifecycle;

use context::Context;
use holochain_wasm_utils::{HcApiReturnCode, SinglePageAllocation};

use action::ActionWrapper;
use instance::Observer;
use nucleus::ribosome::api::ZomeAPIFunction;
use std::{str::FromStr, sync::mpsc::Sender};

use nucleus::memory::*;

use wasmi::{
    self, Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind,
    ValueType,
};

use std::sync::Arc;

pub trait Defn: FromStr {
    /// return the canonical name of this function definition
    fn as_str(&self) -> &'static str;

    /// convert the canonical name of this function to an index
    fn str_index(s: &str) -> usize;

    /// convert an index to the function definition
    fn from_index(i: usize) -> Self;

    // @TODO how to add something to trait that returns functions with unknown params/return?
    // fn as_fn(&self) -> fn(_) -> _;
}

//--------------------------------------------------------------------------------------------------
// Wasm call
//--------------------------------------------------------------------------------------------------

/// Object holding data to pass around to invoked API functions
#[derive(Clone)]
pub struct Runtime {
    context: Arc<Context>,
    pub result: String,
    action_channel: Sender<ActionWrapper>,
    observer_channel: Sender<Observer>,
    memory_manager: SinglePageManager,
}

/// take standard, memory managed runtime argument bytes, extract and convert to serialized struct
pub fn runtime_args_to_utf8(runtime: &Runtime, args: &RuntimeArgs) -> String {
    // @TODO don't panic in WASM
    // @see https://github.com/holochain/holochain-rust/issues/159
    assert_eq!(1, args.len());

    // Read complex argument serialized in memory
    let encoded_allocation: u32 = args.nth(0);
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        .expect("received error instead of valid encoded allocation");
    let bin_arg = runtime.memory_manager.read(allocation);

    // deserialize complex argument
    String::from_utf8(bin_arg)
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        .unwrap()
}

/// given a runtime and a string (e.g. JSON serialized data), allocates bytes and encodes to memory
/// returns a Result suitable to return directly from a zome API function
pub fn runtime_allocate_encode_str(
    runtime: &mut Runtime,
    s: &str,
) -> Result<Option<RuntimeValue>, Trap> {
    // write str to runtime memory
    let mut s_bytes: Vec<_> = s.to_string().into_bytes();
    s_bytes.push(0); // Add string terminate character (important)

    let allocation_of_result = runtime.memory_manager.write(&s_bytes);
    if allocation_of_result.is_err() {
        return Err(Trap::new(TrapKind::MemoryAccessOutOfBounds));
    }

    let encoded_allocation = allocation_of_result
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        .unwrap()
        .encode();

    // Return success in i32 format
    Ok(Some(RuntimeValue::I32(encoded_allocation as i32)))
}

/// Executes an exposed function in a wasm binary
pub fn call(
    context: Arc<Context>,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    wasm: Vec<u8>,
    function_name: &str,
    parameters: Option<Vec<u8>>,
) -> Result<Runtime, InterpreterError> {
    // Create wasm module from wasm binary
    let module = wasmi::Module::from_buffer(wasm).unwrap();

    // Describe invokable functions from within Zome
    impl Externals for Runtime {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            let zf = ZomeAPIFunction::from_index(index);
            match zf {
                ZomeAPIFunction::MissingNo => panic!("unknown function index"),
                _ => zf.as_fn()(self, &args),
            }
        }
    }

    // Define invokable functions from within Zome
    struct RuntimeModuleImportResolver;
    impl ModuleImportResolver for RuntimeModuleImportResolver {
        fn resolve_func(
            &self,
            field_name: &str,
            _signature: &Signature,
        ) -> Result<FuncRef, InterpreterError> {
            let index = ZomeAPIFunction::str_index(&field_name);
            match index {
                index if index == ZomeAPIFunction::MissingNo as usize => {
                    return Err(InterpreterError::Function(format!(
                        "host module doesn't export function with name {}",
                        field_name
                    )));
                }
                _ => Ok(FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                    index as usize,
                )),
            }
        }
    }

    // Create Imports with previously described Resolver
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    // Create module instance from wasm module, and without starting it
    let wasm_instance = ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .assert_no_start();

    // write input arguments for module call in memory Buffer
    let input_parameters: Vec<_> = parameters.unwrap_or_default();

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        context,
        result: String::new(),
        action_channel: action_channel.clone(),
        observer_channel: observer_channel.clone(),
        // memory_manager: ref_memory_manager.clone(),
        memory_manager: SinglePageManager::new(&wasm_instance),
    };

    // scope for mutable borrow of runtime
    let encoded_allocation_of_input: u32;
    {
        let mut_runtime = &mut runtime;
        let allocation_of_input = mut_runtime.memory_manager.write(&input_parameters);
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
                &[RuntimeValue::I32(encoded_allocation_of_input as i32)],
                mut_runtime,
            )?
            .unwrap()
            .try_into()
            .unwrap();
    }

    let allocation_of_output = SinglePageAllocation::new(encoded_allocation_of_output as u32);

    // retrieve invoked wasm function's result that got written in memory
    if let Ok(valid_allocation) = allocation_of_output {
        let result = runtime.memory_manager.read(valid_allocation);
        runtime.result = String::from_utf8(result).unwrap();
    }

    Ok(runtime.clone())
}
