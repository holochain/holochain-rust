// In this example we execute a contract funciton exported as "_call"
mod commit;
mod get;
mod print;

use instance::Observer;
use nucleus::ribosome::{commit::invoke_commit, get::invoke_get, print::invoke_print};
use state;
use std::sync::mpsc::Sender;

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
    ERROR_ACTION_RESULT,
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

pub fn runtime_args_to_utf8(runtime: &Runtime, args: &RuntimeArgs) -> String {
    // @TODO assert or return error?
    // @see https://github.com/holochain/holochain-rust/issues/159
    assert_eq!(2, args.len());

    // Read complex argument serialized in memory
    // @TODO use our Malloced data instead
    // @see https://github.com/holochain/holochain-rust/issues/65

    let mem_offset: u32 = args.nth(0);
    let mem_len: u32 = args.nth(1);
    let bin_arg = runtime
        .memory
        .get(mem_offset, mem_len as usize)
        .expect("Successfully retrieve the arguments");

    String::from_utf8(bin_arg).unwrap()
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

#[cfg(test)]
pub mod tests {
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    extern crate test_utils;
    use super::Runtime;
    use super::call;
    use ::instance::tests::test_instance;

    use holochain_dna::zome::capabilities::ReservedCapabilityNames;


    pub fn test_zome_api_function_wasm(canonical_name: &str) -> Vec<u8> {
        let wasm_binary = Wat2Wasm::new()
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
                // name as the function named `$<canonical name>` in WAT
                // define the signature as 2 inputs, 1 output
                // the signature is the same as the exported "test_get_dispatch" function below as
                // we want the latter to be a thin wrapper for the former
                // (import "env" "<canonical name>"
                //      (func $<canonical name>
                //          (param i32)
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
                // define and export the *_dispatch function that will be called from the
                // ribosome rust implementation, where * is the fourth arg to `call`
                // @see nucleus::ribosome::call
                // (func (export "*_dispatch") ...)
                //
                // define the memory offset and length that the serialized input struct can be
                // found across as params to the exported function, also the function return type
                // (param $offset i32)
                // (param $length i32)
                // (result i32)
                //
                // call the imported function and pass the exported function arguments straight
                // through, let the return also fall straight through
                // `get_local` maps the relevant arguments in the local scope
                // (call
                //      $<canonical name>
                //      (get_local $offset)
                //      (get_local $length)
                // )
                format!(r#"
(module
    (import "env" "{}"
        (func $zome_api_function
            (param i32)
            (param i32)
            (result i32)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "test_dispatch")
            (param $offset i32)
            (param $length i32)
            (result i32)

        (call
            $zome_api_function
            (get_local $offset)
            (get_local $length)
        )
    )
)
                "#, canonical_name),
            )
            .unwrap();

        wasm_binary.as_ref().to_vec()
    }

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
