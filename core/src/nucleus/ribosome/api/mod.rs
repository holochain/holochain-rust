//! Module for ZomeApiFunctions
//! ZomeApiFunctions are the functions provided by the ribosome that are callable by Zomes.

pub mod call;
pub mod commit;
pub mod debug;
pub mod get;
pub mod get_links;
pub mod init_globals;
use action::ActionWrapper;
use context::Context;
use holochain_dna::zome::capabilities::ReservedCapabilityNames;
use holochain_wasm_utils::{HcApiReturnCode, SinglePageAllocation};
use instance::Observer;
use nucleus::{
    memory::SinglePageManager,
    ribosome::{
        api::{
            call::invoke_call, commit::invoke_commit_entry, debug::invoke_debug,
            get::invoke_get_entry, init_globals::invoke_init_globals,
        },
        Defn,
    },
    ZomeFnCall,
};
use num_traits::FromPrimitive;
use std::{
    str::FromStr,
    sync::{mpsc::Sender, Arc},
};
use wasmi::{
    self, Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, NopExternals, RuntimeArgs, RuntimeValue, Signature, Trap,
    TrapKind, ValueType,
};

//--------------------------------------------------------------------------------------------------
// ZOME API FUNCTION DEFINITIONS
//--------------------------------------------------------------------------------------------------

/// Enumeration of all the Zome Functions known and usable in Zomes.
/// Enumeration can convert to str.
#[repr(usize)]
#[derive(FromPrimitive, Debug, PartialEq, Eq)]
pub enum ZomeApiFunction {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// Abort is a way to receive useful debug info from
    /// assemblyscript memory allocators
    /// message: mem address in the wasm memory for an error message
    /// filename: mem address in the wasm memory for a filename
    /// line: line number
    /// column: column number
    Abort,

    /// Zome API

    /// send debug information to the log
    /// debug(s: String)
    Debug,

    /// Commit an app entry to source chain
    /// commit_entry(entry_type: String, entry_content: String) -> Hash
    CommitAppEntry,

    /// Get an app entry from source chain by key (header hash)
    /// get_entry(key: String) -> Pair
    GetAppEntry,

    /// Init App Globals
    /// hc_init_globals() -> InitGlobalsOutput
    InitGlobals,

    /// Call a zome function in a different capability or zome
    /// hc_call(zome_name: String, cap_name: String, fn_name: String, args: String);
    Call,
}

impl Defn for ZomeApiFunction {
    fn as_str(&self) -> &'static str {
        match *self {
            ZomeApiFunction::MissingNo => "",
            ZomeApiFunction::Abort => "abort",
            ZomeApiFunction::Debug => "hc_debug",
            ZomeApiFunction::CommitAppEntry => "hc_commit_entry",
            ZomeApiFunction::GetAppEntry => "hc_get_entry",
            ZomeApiFunction::InitGlobals => "hc_init_globals",
            ZomeApiFunction::Call => "hc_call",
        }
    }

    fn str_to_index(s: &str) -> usize {
        match ZomeApiFunction::from_str(s) {
            Ok(i) => i as usize,
            Err(_) => ZomeApiFunction::MissingNo as usize,
        }
    }

    fn from_index(i: usize) -> Self {
        match FromPrimitive::from_usize(i) {
            Some(v) => v,
            None => ZomeApiFunction::MissingNo,
        }
    }

    fn capability(&self) -> ReservedCapabilityNames {
        // Zome API Functions are not part of any zome and capability
        // @TODO architecture issue?
        // @see https://github.com/holochain/holochain-rust/issues/299
        unreachable!();
    }
}

impl FromStr for ZomeApiFunction {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abort" => Ok(ZomeApiFunction::Abort),
            "hc_debug" => Ok(ZomeApiFunction::Debug),
            "hc_commit_entry" => Ok(ZomeApiFunction::CommitAppEntry),
            "hc_get_entry" => Ok(ZomeApiFunction::GetAppEntry),
            "hc_init_globals" => Ok(ZomeApiFunction::InitGlobals),
            "hc_call" => Ok(ZomeApiFunction::Call),
            _ => Err("Cannot convert string to ZomeApiFunction"),
        }
    }
}

impl ZomeApiFunction {
    pub fn as_fn(&self) -> (fn(&mut Runtime, &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap>) {
        /// does nothing, escape hatch so the compiler can enforce exhaustive matching below
        fn noop(_runtime: &mut Runtime, _args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
            Ok(Some(RuntimeValue::I32(0 as i32)))
        }

        // TODO Implement a proper "abort" function for handling assemblyscript aborts
        // @see: https://github.com/holochain/holochain-rust/issues/324

        match *self {
            ZomeApiFunction::MissingNo => noop,
            ZomeApiFunction::Abort => noop,
            ZomeApiFunction::Debug => invoke_debug,
            ZomeApiFunction::CommitAppEntry => invoke_commit_entry,
            ZomeApiFunction::GetAppEntry => invoke_get_entry,
            ZomeApiFunction::InitGlobals => invoke_init_globals,
            ZomeApiFunction::Call => invoke_call,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Wasm call
//--------------------------------------------------------------------------------------------------

/// Object holding data to pass around to invoked Zome API functions
#[derive(Clone)]
pub struct Runtime {
    pub context: Arc<Context>,
    pub result: String,
    action_channel: Sender<ActionWrapper>,
    observer_channel: Sender<Observer>,
    memory_manager: SinglePageManager,
    zome_call: ZomeFnCall,
    pub app_name: String,
}

impl Runtime {
    /// Load a string stored in wasm memory.
    /// Input RuntimeArgs should only have one input which is the encoded allocation holding
    /// the complex data as an utf8 string.
    /// Returns the utf8 string.
    pub fn load_utf8_from_args(&self, args: &RuntimeArgs) -> String {
        // @TODO don't panic in WASM
        // @see https://github.com/holochain/holochain-rust/issues/159
        assert_eq!(1, args.len());

        // Read complex argument serialized in memory
        let encoded_allocation: u32 = args.nth(0);
        let allocation = SinglePageAllocation::new(encoded_allocation);
        // Handle empty allocation edge case
        if let Err(HcApiReturnCode::Success) = allocation {
            return String::new();
        }
        let allocation = allocation
            // @TODO don't panic in WASM
            // @see https://github.com/holochain/holochain-rust/issues/159
            .expect("received error instead of valid encoded allocation");
        let bin_arg = self.memory_manager.read(allocation);

        // convert complex argument
        String::from_utf8(bin_arg)
            // @TODO don't panic in WASM
            // @see https://github.com/holochain/holochain-rust/issues/159
            .unwrap()
    }

    /// Store a string in wasm memory.
    /// Input should be a a json string.
    /// Returns a Result suitable to return directly from a zome API function, i.e. an encoded allocation
    pub fn store_utf8(&mut self, json_str: &str) -> Result<Option<RuntimeValue>, Trap> {
        // write str to runtime memory
        let mut s_bytes: Vec<_> = json_str.to_string().into_bytes();
        s_bytes.push(0); // Add string terminate character (important)

        let allocation_of_result = self.memory_manager.write(&s_bytes);
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
}

/// Executes an exposed function in a wasm binary
/// Multithreaded function
/// panics if wasm isn't valid
pub fn call(
    app_name: &str,
    context: Arc<Context>,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
    wasm: Vec<u8>,
    zome_call: &ZomeFnCall,
    parameters: Option<Vec<u8>>,
) -> Result<Runtime, InterpreterError> {
    // Create wasm module from wasm binary
    let module = wasmi::Module::from_buffer(wasm).expect("wasm should be valid");

    // invoke_index and resolve_func work together to enable callable host functions
    // within WASM modules, which is how the core API functions
    // read about the Externals trait for more detail

    // Correlate the indexes of core API functions with a call to the actual function
    // by implementing the Externals wasmi trait for Runtime
    impl Externals for Runtime {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            let zf = ZomeApiFunction::from_index(index);
            match zf {
                ZomeApiFunction::MissingNo => panic!("unknown function index"),
                // convert the function to its callable form and call it with the given arguments
                _ => zf.as_fn()(self, &args),
            }
        }
    }

    // Correlate the names of the core ZomeApiFunction's with their indexes
    // and declare its function signature (which is always the same)
    struct RuntimeModuleImportResolver;
    impl ModuleImportResolver for RuntimeModuleImportResolver {
        fn resolve_func(
            &self,
            field_name: &str,
            _signature: &Signature,
        ) -> Result<FuncRef, InterpreterError> {
            let api_fn = match ZomeApiFunction::from_str(&field_name) {
                Ok(api_fn) => api_fn,
                Err(_) => {
                    return Err(InterpreterError::Function(format!(
                        "host module doesn't export function with name {}",
                        field_name
                    )));
                }
            };

            match api_fn {
                // Abort is a way to receive useful debug info from
                // assemblyscript memory allocators, see enum definition for function signature
                ZomeApiFunction::Abort => Ok(FuncInstance::alloc_host(
                    Signature::new(
                        &[
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ][..],
                        None,
                    ),
                    api_fn as usize,
                )),
                // All of our Zome API Functions have the same signature
                _ => Ok(FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                    api_fn as usize,
                )),
            }
        }
    }

    // Create Imports with previously described Resolver
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    // Create module instance from wasm module, and start it if start is defined
    let wasm_instance = ModuleInstance::new(&module, &imports)
        .expect("Failed to instantiate module")
        .run_start(&mut NopExternals)?;

    // write input arguments for module call in memory Buffer
    let input_parameters: Vec<_> = parameters.unwrap_or_default();

    // instantiate runtime struct for passing external state data over wasm but not to wasm
    let mut runtime = Runtime {
        context,
        result: String::new(),
        action_channel: action_channel.clone(),
        observer_channel: observer_channel.clone(),
        memory_manager: SinglePageManager::new(&wasm_instance),
        zome_call: zome_call.clone(),
        app_name: app_name.to_string(),
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
                zome_call.fn_name.clone().as_str(),
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
    extern crate holochain_agent;
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    extern crate test_utils;
    use super::ZomeApiFunction;
    use context::Context;
    use instance::{
        tests::{test_context_and_logger, TestLogger},
        Instance,
    };
    use nucleus::{
        ribosome::{
            api::{call, Runtime},
            callback::{tests::test_callback_instance, Callback},
            Defn,
        },
        ZomeFnCall,
    };
    use std::{
        str::FromStr,
        sync::{Arc, Mutex},
    };

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
                // define and export the test function that will be called from the
                // ribosome rust implementation, where "test" is the fourth arg to `call`
                // @see `test_zome_api_function_runtime`
                // @see nucleus::ribosome::call
                // (func (export "test") ...)
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
        (export "test")
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

    /// dummy zome name
    pub fn test_zome_name() -> String {
        "test_zome".to_string()
    }

    /// dummy capability
    pub fn test_capability() -> String {
        ReservedCapabilityNames::MissingNo.as_str().to_string()
    }

    /// dummy zome API function name
    pub fn test_function_name() -> String {
        "test".to_string()
    }

    /// dummy parameters for a zome API function call
    pub fn test_parameters() -> String {
        String::new()
    }

    /// calls the zome API function with passed bytes argument using the instance runtime
    /// returns the runtime after the call completes
    pub fn test_zome_api_function_call(
        app_name: &str,
        context: Arc<Context>,
        logger: Arc<Mutex<TestLogger>>,
        instance: &Instance,
        wasm: &Vec<u8>,
        args_bytes: Vec<u8>,
    ) -> (Runtime, Arc<Mutex<TestLogger>>) {
        let zome_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            &test_function_name(),
            &test_parameters(),
        );
        (
            call(
                &app_name,
                context,
                &instance.action_channel(),
                &instance.observer_channel(),
                wasm.clone(),
                &zome_call,
                Some(args_bytes),
            ).expect("test should be callable"),
            logger,
        )
    }

    /// given a canonical zome API function name and args as bytes:
    /// - builds wasm with test_zome_api_function_wasm
    /// - builds dna and test instance
    /// - calls the zome API function with passed bytes argument using the instance runtime
    /// - returns the runtime after the call completes
    pub fn test_zome_api_function_runtime(
        canonical_name: &str,
        args_bytes: Vec<u8>,
    ) -> (Runtime, Arc<Mutex<TestLogger>>) {
        let wasm = test_zome_api_function_wasm(canonical_name);
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );
        //let instance = test_instance(dna.clone());
        let instance =
            test_callback_instance(&test_zome_name(), Callback::ValidateCommit.as_str(), 0);

        let (c, logger) = test_context_and_logger("joan");
        let context = instance.initialize_context(c);

        test_zome_api_function_call(
            &dna.name.to_string(),
            context,
            logger,
            &instance,
            &wasm,
            args_bytes,
        )
    }

    #[test]
    /// test the FromStr implementation for ZomeApiFunction
    fn test_from_str() {
        assert_eq!(
            ZomeApiFunction::Debug,
            ZomeApiFunction::from_str("hc_debug").unwrap(),
        );
        assert_eq!(
            ZomeApiFunction::CommitAppEntry,
            ZomeApiFunction::from_str("hc_commit_entry").unwrap(),
        );
        assert_eq!(
            ZomeApiFunction::GetAppEntry,
            ZomeApiFunction::from_str("hc_get_entry").unwrap(),
        );

        assert_eq!(
            "Cannot convert string to ZomeApiFunction",
            ZomeApiFunction::from_str("foo").unwrap_err(),
        );
    }

}
