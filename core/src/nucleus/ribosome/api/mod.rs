pub mod commit;
pub mod debug;
pub mod get;

use nucleus::ribosome::{
    api::{commit::invoke_commit, debug::invoke_debug, get::invoke_get},
    Defn, Runtime,
};
use num_traits::FromPrimitive;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

// Zome API functions are exposed by HC to zome logic

//--------------------------------------------------------------------------------------------------
// ZOME API FUNCTION DEFINITIONS
//--------------------------------------------------------------------------------------------------

/// Enumeration of all Zome functions known and used by HC Core
/// Enumeration converts to str
#[repr(usize)]
#[derive(FromPrimitive)]
pub enum ZomeAPIFunction {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// Zome API

    /// send debug information to the log
    /// debug(s : String)
    Debug,

    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String) -> Hash
    Commit,

    /// Get an entry from source chain by key (header hash)
    /// get(key: String) -> Pair
    Get,
}

impl Defn for ZomeAPIFunction {
    fn as_str(&self) -> &'static str {
        match *self {
            ZomeAPIFunction::MissingNo => "",
            ZomeAPIFunction::Debug => "debug",
            ZomeAPIFunction::Commit => "commit",
            ZomeAPIFunction::Get => "get",
        }
    }

    fn str_index(s: &str) -> usize {
        match ZomeAPIFunction::from_str(s) {
            Ok(i) => i as usize,
            Err(_) => ZomeAPIFunction::MissingNo as usize,
        }
    }

    fn from_index(i: usize) -> Self {
        match FromPrimitive::from_usize(i) {
            Some(v) => v,
            None => ZomeAPIFunction::MissingNo,
        }
    }
}

impl FromStr for ZomeAPIFunction {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(ZomeAPIFunction::Debug),
            "commit" => Ok(ZomeAPIFunction::Commit),
            "get" => Ok(ZomeAPIFunction::Get),
            _ => Err("Cannot convert string to ZomeAPIFunction"),
        }
    }
}

impl ZomeAPIFunction {
    pub fn as_fn(&self) -> (fn(&mut Runtime, &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap>) {
        /// does nothing, escape hatch so the compiler can enforce exhaustive matching below
        fn noop(_runtime: &mut Runtime, _args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
            Ok(Some(RuntimeValue::I32(0 as i32)))
        }

        match *self {
            ZomeAPIFunction::MissingNo => noop,
            ZomeAPIFunction::Debug => invoke_debug,
            ZomeAPIFunction::Commit => invoke_commit,
            ZomeAPIFunction::Get => invoke_get,
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate holochain_agent;
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    extern crate test_utils;
    use nucleus::ribosome::{call, Runtime};
    use instance::tests::{test_context_and_logger, test_instance, TestLogger};
    use std::sync::{Arc, Mutex};

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
    pub fn test_zome_api_function_runtime(
        canonical_name: &str,
        args_bytes: Vec<u8>,
    ) -> (Runtime, Arc<Mutex<TestLogger>>) {
        let wasm = test_zome_api_function_wasm(canonical_name);
        let dna = test_utils::create_test_dna_with_wasm(
            "test_zome".into(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            wasm.clone(),
        );
        let instance = test_instance(dna);
        let (context, logger) = test_context_and_logger("joan");
        (
            call(
                context,
                &instance.action_channel(),
                &instance.observer_channel(),
                wasm.clone(),
                "test",
                Some(args_bytes),
            ).expect("test should be callable"),
            logger,
        )
    }

}
