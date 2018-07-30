mod commit;
mod get;
mod print;

use holochain_wasm_utils::{HcApiReturnCode, SinglePageAllocation};

use instance::Observer;
use nucleus::ribosome::{commit::invoke_commit, get::invoke_get, print::invoke_print};
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
    /// Error index for unimplemented functions
    MissingNo = 0,
    /// Print debug information in the console
    /// print(s : String)
    Print,
    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String) -> Hash
    Commit,
    /// Get an entry from source chain by key (header hash)
    /// get(key: String) -> Pair
    Get,
    // Add new API function index here
    // ...
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

/// maps canonical zome API function names to an HcApiFuncIndex variant
fn index_canonical_name(canonical_name: &str) -> HcApiFuncIndex {
    match canonical_name {
        "print" => HcApiFuncIndex::Print,
        "commit" => HcApiFuncIndex::Commit,
        "get" => HcApiFuncIndex::Get,
        _ => HcApiFuncIndex::MissingNo,
    }
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
            // @TODO don't maintain this list manually
            // @see https://github.com/holochain/holochain-rust/issues/171
            match index {
                index if index == HcApiFuncIndex::PRINT as usize => invoke_print(self, &args),
                index if index == HcApiFuncIndex::COMMIT as usize => invoke_commit(self, &args),
                index if index == HcApiFuncIndex::GET as usize => invoke_get(self, &args),
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
            let index = index_canonical_name(field_name);
            match index {
                HcApiFuncIndex::MissingNo => {
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
        print_output: String::new(),
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

#[cfg(test)]
pub mod tests {
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    extern crate test_utils;
    use super::{call, Runtime};
    use instance::tests::test_instance;

    use holochain_dna::zome::capabilities::ReservedCapabilityNames;

    /// generates the wasm to dispatch any zome API function with a single memomry managed runtime
    /// and bytes argument
    pub fn test_zome_api_function_wasm(canonical_name: &str) -> Vec<u8> {
        Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                // We don't expect everyone to be a pro at hand-coding WAT so here's a "how to".
                // WAT does not have comments so code is duplicated in the comments here.
                //
                // How this works:
                //
                // root of the s-expression tree
                // (module ...)
                //
                // imports must be the first expressions in a module
                // imports the fn from the rust environment using its canonical zome API function
                // name as the function named `$zome_api_function` in WAT
                // define the signature as 1 input, 1 output
                // the signature is the same as the exported "test_dispatch" function below as
                // we want the latter to be a thin wrapper for the former
                // (import "env" "<canonical name>"
                //      (func $zome_api_function
                //          (param i32)
                //          (result i32)
                //      )
                // )
                //
                // only need 1 page of memory for testing
                // (memory 1)
                //
                // all modules compiled with rustc must have an export named "memory" (or fatal)
                // (export "memory" (memory 0))
                //
                // define and export the test_dispatch function that will be called from the
                // ribosome rust implementation, where "test" is the fourth arg to `call`
                // @see `test_zome_api_function_runtime`
                // @see nucleus::ribosome::call
                // (func (export "test_dispatch") ...)
                //
                // define the memory allocation for the memory manager that the serialized input
                // struct can be found across as an i32 to the exported function, also the function
                // return type is i32
                // (param $allocation i32)
                // (result i32)
                //
                // call the imported function and pass the exported function arguments straight
                // through, let the return also fall straight through
                // `get_local` maps the relevant arguments in the local scope
                // (call
                //      $zome_api_function
                //      (get_local $allocation)
                // )
                format!(
                    r#"
(module
    (import "env" "{}"
        (func $zome_api_function
            (param i32)
            (result i32)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "test_dispatch")
            (param $allocation i32)
            (result i32)

        (call
            $zome_api_function
            (get_local $allocation)
        )
    )
)
                "#,
                    canonical_name
                ),
            )
            .unwrap()
            .as_ref()
            .to_vec()
    }

    /// given a canonical zome API function name and args as bytes:
    /// - builds wasm with test_zome_api_function_wasm
    /// - builds dna and test instance
    /// - calls the zome API function with passed bytes argument using the instance runtime
    /// - returns the runtime after the call completes
    pub fn test_zome_api_function_runtime(canonical_name: &str, args_bytes: Vec<u8>) -> Runtime {
        let wasm = test_zome_api_function_wasm(canonical_name);
        let dna = test_utils::create_test_dna_with_wasm(
            "test_zome".into(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            wasm.clone(),
        );
        let instance = test_instance(dna);

        call(
            &instance.action_channel(),
            &instance.observer_channel(),
            wasm.clone(),
            "test",
            Some(args_bytes),
        ).expect("test should be callable")
    }

}
