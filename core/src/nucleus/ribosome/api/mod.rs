//! Module for ZomeApiFunctions
//! ZomeApiFunctions are the functions provided by the ribosome that are callable by Zomes.

pub mod call;
pub mod commit;
pub mod debug;
pub mod get_entry;
pub mod get_links;
pub mod hash_entry;
pub mod init_globals;
pub mod link_entries;
pub mod query;

use holochain_dna::zome::capabilities::ReservedCapabilityNames;
use nucleus::ribosome::{
    api::{
        call::invoke_call, commit::invoke_commit_app_entry, debug::invoke_debug,
        get_entry::invoke_get_entry, get_links::invoke_get_links, hash_entry::invoke_hash_entry,
        init_globals::invoke_init_globals, link_entries::invoke_link_entries, query::invoke_query,
    },
    Defn, Runtime,
};
use num_traits::FromPrimitive;
use std::str::FromStr;

use wasmi::{RuntimeArgs, RuntimeValue, Trap};

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
    /// commit_entry(entry_type: String, entry_value: String) -> Hash
    CommitAppEntry,

    /// Get an app entry from source chain by key (header hash)
    /// get_entry(address: Address) -> Entry
    GetAppEntry,

    /// Init Zome API Globals
    /// hc_init_globals() -> InitGlobalsOutput
    InitGlobals,

    /// Call a zome function in a different capability or zome
    /// hc_call(zome_name: String, cap_name: String, fn_name: String, args: String);
    Call,

    LinkEntries,
    GetLinks,
    Query,
    HashEntry,
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
            ZomeApiFunction::LinkEntries => "hc_link_entries",
            ZomeApiFunction::GetLinks => "hc_get_links",
            ZomeApiFunction::Query => "hc_query",
            ZomeApiFunction::HashEntry => "hc_hash_entry",
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
            "hc_link_entries" => Ok(ZomeApiFunction::LinkEntries),
            "hc_get_links" => Ok(ZomeApiFunction::GetLinks),
            "hc_query" => Ok(ZomeApiFunction::Query),
            "hc_hash_entry" => Ok(ZomeApiFunction::HashEntry),
            _ => Err("Cannot convert string to ZomeApiFunction"),
        }
    }
}

/// does nothing, escape hatch so the compiler can enforce exhaustive matching in as_fn
fn noop(_runtime: &mut Runtime, _args: &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
    // Return Ribosome Success Code
    Ok(Some(RuntimeValue::I32(0 as i32)))
}

impl ZomeApiFunction {
    // cannot test this because PartialEq is not implemented for fns
    #[cfg_attr(tarpaulin, skip)]
    pub fn as_fn(&self) -> (fn(&mut Runtime, &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap>) {
        // TODO Implement a proper "abort" function for handling assemblyscript aborts
        // @see: https://github.com/holochain/holochain-rust/issues/324

        match *self {
            ZomeApiFunction::MissingNo => noop,
            ZomeApiFunction::Abort => noop,
            ZomeApiFunction::Debug => invoke_debug,
            ZomeApiFunction::CommitAppEntry => invoke_commit_app_entry,
            ZomeApiFunction::GetAppEntry => invoke_get_entry,
            ZomeApiFunction::InitGlobals => invoke_init_globals,
            ZomeApiFunction::Call => invoke_call,
            ZomeApiFunction::LinkEntries => invoke_link_entries,
            ZomeApiFunction::GetLinks => invoke_get_links,
            ZomeApiFunction::Query => invoke_query,
            ZomeApiFunction::HashEntry => invoke_hash_entry,
        }
    }
}

#[cfg(test)]
pub mod tests {
    extern crate holochain_agent;
    extern crate wabt;
    use self::wabt::Wat2Wasm;
    extern crate test_utils;
    use super::ZomeApiFunction;
    use context::Context;
    use instance::{tests::test_instance_and_context, Instance};
    use nucleus::{
        ribosome::{self, Defn},
        ZomeFnCall,
    };
    use std::{str::FromStr, sync::Arc};

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

    (func
        (export "__hdk_validate_app_entry")
        (param $allocation i32)
        (result i32)

        (i32.const 0)
    )


    (func
        (export "__hdk_get_validation_package_for_entry_type")
        (param $allocation i32)
        (result i32)

        ;; This writes "Entry" into memory
        (i32.store (i32.const 0) (i32.const 34))
        (i32.store (i32.const 1) (i32.const 69))
        (i32.store (i32.const 2) (i32.const 110))
        (i32.store (i32.const 3) (i32.const 116))
        (i32.store (i32.const 4) (i32.const 114))
        (i32.store (i32.const 5) (i32.const 121))
        (i32.store (i32.const 6) (i32.const 34))

        (i32.const 7)
    )

    (func
        (export "__list_capabilities")
        (param $allocation i32)
        (result i32)

        (i32.const 0)
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
        dna_name: &str,
        context: Arc<Context>,
        _instance: &Instance,
        wasm: &Vec<u8>,
        args_bytes: Vec<u8>,
    ) -> String {
        let zome_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            &test_function_name(),
            &test_parameters(),
        );
        ribosome::run_dna(
            &dna_name,
            context,
            wasm.clone(),
            &zome_call,
            Some(args_bytes),
        )
        .expect("test should be callable")
    }

    /// Given a canonical zome API function name and args as bytes:
    /// - builds wasm with test_zome_api_function_wasm
    /// - builds dna and test instance
    /// - calls the zome API function with passed bytes argument using the instance runtime
    /// - returns the call result
    pub fn test_zome_api_function(
        canonical_name: &str,
        args_bytes: Vec<u8>,
    ) -> (String, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(canonical_name);
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );

        let dna_name = &dna.name.to_string().clone();
        let (instance, context) =
            test_instance_and_context(dna).expect("Could not create test instance");

        let call_result =
            test_zome_api_function_call(&dna_name, context.clone(), &instance, &wasm, args_bytes);
        (call_result, context)
    }

    #[test]
    /// test the FromStr implementation for ZomeApiFunction
    fn test_from_str() {
        for (input, output) in vec![
            ("abort", ZomeApiFunction::Abort),
            ("hc_debug", ZomeApiFunction::Debug),
            ("hc_commit_entry", ZomeApiFunction::CommitAppEntry),
            ("hc_get_entry", ZomeApiFunction::GetAppEntry),
            ("hc_init_globals", ZomeApiFunction::InitGlobals),
            ("hc_call", ZomeApiFunction::Call),
            ("hc_link_entries", ZomeApiFunction::LinkEntries),
            ("hc_get_links", ZomeApiFunction::GetLinks),
            ("hc_query", ZomeApiFunction::Query),
            ("hc_hash_entry", ZomeApiFunction::HashEntry),
        ] {
            assert_eq!(ZomeApiFunction::from_str(input).unwrap(), output);
        }

        assert_eq!(
            "Cannot convert string to ZomeApiFunction",
            ZomeApiFunction::from_str("foo").unwrap_err(),
        );
    }

    #[test]
    /// Show Defn implementation
    fn defn_test() {
        // as_str()
        for (input, output) in vec![
            (ZomeApiFunction::MissingNo, ""),
            (ZomeApiFunction::Abort, "abort"),
            (ZomeApiFunction::Debug, "hc_debug"),
            (ZomeApiFunction::CommitAppEntry, "hc_commit_entry"),
            (ZomeApiFunction::GetAppEntry, "hc_get_entry"),
            (ZomeApiFunction::InitGlobals, "hc_init_globals"),
            (ZomeApiFunction::Call, "hc_call"),
            (ZomeApiFunction::LinkEntries, "hc_link_entries"),
            (ZomeApiFunction::GetLinks, "hc_get_links"),
            (ZomeApiFunction::Query, "hc_query"),
            (ZomeApiFunction::HashEntry, "hc_hash_entry"),
        ] {
            assert_eq!(output, input.as_str());
        }

        // str_to_index()
        for (input, output) in vec![
            ("", 0),
            ("abort", 1),
            ("hc_debug", 2),
            ("hc_commit_entry", 3),
            ("hc_get_entry", 4),
            ("hc_init_globals", 5),
            ("hc_call", 6),
            ("hc_link_entries", 7),
            ("hc_get_links", 8),
            ("hc_query", 9),
            ("hc_hash_entry", 10),
        ] {
            assert_eq!(output, ZomeApiFunction::str_to_index(input));
        }

        // from_index()
        for (input, output) in vec![
            (0, ZomeApiFunction::MissingNo),
            (1, ZomeApiFunction::Abort),
            (2, ZomeApiFunction::Debug),
            (3, ZomeApiFunction::CommitAppEntry),
            (4, ZomeApiFunction::GetAppEntry),
            (5, ZomeApiFunction::InitGlobals),
            (6, ZomeApiFunction::Call),
            (7, ZomeApiFunction::LinkEntries),
            (8, ZomeApiFunction::GetLinks),
            (9, ZomeApiFunction::Query),
            (10, ZomeApiFunction::HashEntry),
        ] {
            assert_eq!(output, ZomeApiFunction::from_index(input));
        }
    }

}
