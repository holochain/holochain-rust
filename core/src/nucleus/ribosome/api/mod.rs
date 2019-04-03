//! Module for ZomeApiFunctions
//! ZomeApiFunctions are the functions provided by the ribosome that are callable by Zomes.

pub mod call;
pub mod commit;
pub mod debug;
pub mod entry_address;
pub mod get_entry;
pub mod get_links;
pub mod init_globals;
pub mod link_entries;
#[macro_use]
mod macros;
pub mod keystore;
pub mod query;
pub mod remove_entry;
pub mod remove_link;
pub mod send;
pub mod sign;
pub mod sleep;
pub mod update_entry;
pub mod verify_signature;

use crate::nucleus::ribosome::{
    api::{
        call::invoke_call,
        commit::invoke_commit_app_entry,
        debug::invoke_debug,
        entry_address::invoke_entry_address,
        get_entry::invoke_get_entry,
        get_links::invoke_get_links,
        init_globals::invoke_init_globals,
        keystore::{
            invoke_keystore_derive_key, invoke_keystore_derive_seed, invoke_keystore_list,
            invoke_keystore_new_random, invoke_keystore_sign,
        },
        link_entries::invoke_link_entries,
        query::invoke_query,
        remove_entry::invoke_remove_entry,
        remove_link::invoke_remove_link,
        send::invoke_send,
        sign::{invoke_sign, invoke_sign_one_time},
        sleep::invoke_sleep,
        update_entry::invoke_update_entry,
        verify_signature::invoke_verify_signature,
    },
    runtime::Runtime,
};

use crate::nucleus::ribosome::Defn;
use num_traits::FromPrimitive;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

pub type ZomeApiResult = Result<Option<RuntimeValue>, Trap>;

//--------------------------------------------------------------------------------------------------
// ZOME API FUNCTION DEFINITIONS
//--------------------------------------------------------------------------------------------------

link_zome_api! {
    /// send debug information to the log
    /// debug(s: String)
    "hc_debug", Debug, invoke_debug;

    /// Commit an app entry to source chain
    /// commit_entry(entry_type: String, entry_value: String) -> Address
    "hc_commit_entry", CommitAppEntry, invoke_commit_app_entry;

    /// Get an app entry from source chain by key (header hash)
    /// get_entry(address: Address) -> Entry
    "hc_get_entry", GetAppEntry, invoke_get_entry;
    "hc_update_entry", UpdateEntry, invoke_update_entry;
    "hc_remove_entry", RemoveEntry, invoke_remove_entry;

    /// Init Zome API Globals
    /// hc_init_globals() -> InitGlobalsOutput
    "hc_init_globals", InitGlobals, invoke_init_globals;

    /// Call a zome function in a different zome or dna via a bridge
    /// hc_call(zome_name: String, cap_token: Address, fn_name: String, args: String);
    "hc_call", Call, invoke_call;

    /// Create a link entry
    "hc_link_entries", LinkEntries, invoke_link_entries;

    /// Retrieve links from the DHT
    "hc_get_links", GetLinks, invoke_get_links;

    /// Query the local chain for entries
    "hc_query", Query, invoke_query;

    /// Pass an entry to retrieve its address
    /// the address algorithm is specific to the entry, typically sha256 but can differ
    /// entry_address(entry: Entry) -> Address
    "hc_entry_address", EntryAddress, invoke_entry_address;

    /// Send a message directly to another node
    "hc_send", Send, invoke_send;

    /// Allow a specified amount of time to pass
    "hc_sleep", Sleep, invoke_sleep;

    /// Commit link deletion entry
    "hc_remove_link", RemoveLink, invoke_remove_link;

    /// Sign a block of data with the Agent key
    "hc_sign", Sign, invoke_sign;

    /// Sign a block of data with a one-time key that is then shredded
    "hc_sign_one_time", SignOneTime, invoke_sign_one_time;

    /// Verify that a block of data was signed by a given public key
    "hc_verify_signature", VerifySignature, invoke_verify_signature;

    /// Retrieve a list of identifiers of the secrets in the keystore
    "hc_keystore_list", KeystoreList, invoke_keystore_list;

    /// Create a new random seed Secret in the keystore
    "hc_keystore_new_random", KeystoreNewRandom, invoke_keystore_new_random;

    /// Derive a new seed from an existing seed in the keystore
    "hc_keystore_derive_seed", KeystoreDeriveSeed, invoke_keystore_derive_seed;

    /// Create a new key (signing or encrypting) as derived from an existing seed in the keystore
    "hc_keystore_derive_key", KeystoreDeriveKey, invoke_keystore_derive_key;

    /// Sign a block of data using a key in the keystore
    "hc_keystore_sign", KeystoreSign, invoke_keystore_sign;
}

#[cfg(test)]
pub mod tests {
    use self::wabt::Wat2Wasm;
    use crate::{
        context::Context,
        instance::{tests::test_instance_and_context, Instance},
        nucleus::{
            ribosome::{self, runtime::WasmCallData},
            tests::test_capability_request,
            ZomeFnCall,
        },
    };
    use holochain_core_types::json::JsonString;
    use std::sync::Arc;
    use test_utils;
    use wabt;

    /// generates the wasm to dispatch any zome API function with a single memory managed runtime
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
                //          (param i64)
                //          (result i64)
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
                // struct can be found across as an i64 to the exported function, also the function
                // return type is i64
                // (param $allocation i64)
                // (result i64)
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
            (param i64)
            (result i64)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "test")
            (param $allocation i64)
            (result i64)

        (call
            $zome_api_function
            (get_local $allocation)
        )
    )

    (func
        (export "__hdk_validate_app_entry")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__hdk_validate_link")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )


    (func
        (export "__hdk_get_validation_package_for_entry_type")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__hdk_get_validation_package_for_link")
        (param $allocation i64)
        (result i64)

        ;; This writes "Entry" into memory
        (i64.store (i32.const 0) (i64.const 34))
        (i64.store (i32.const 1) (i64.const 69))
        (i64.store (i32.const 2) (i64.const 110))
        (i64.store (i32.const 3) (i64.const 116))
        (i64.store (i32.const 4) (i64.const 114))
        (i64.store (i32.const 5) (i64.const 121))
        (i64.store (i32.const 6) (i64.const 34))

        (i64.const 7)
    )

    (func
        (export "__list_traits")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
    )

    (func
        (export "__list_functions")
        (param $allocation i64)
        (result i64)

        (i64.const 0)
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

    /// dummy zome API function name
    pub fn test_function_name() -> String {
        "test".to_string()
    }

    /// dummy parameters for a zome API function call
    pub fn test_parameters() -> JsonString {
        JsonString::empty_object()
    }

    /// calls the zome API function with passed bytes argument using the instance runtime
    /// returns the runtime after the call completes
    pub fn test_zome_api_function_call(
        dna_name: &str,
        context: Arc<Context>,
        _instance: &Instance,
        wasm: &Vec<u8>,
        args_bytes: Vec<u8>,
    ) -> JsonString {
        let zome_call = ZomeFnCall::new(
            &test_zome_name(),
            test_capability_request(context.clone(), &test_function_name(), test_parameters()),
            &test_function_name(),
            test_parameters(),
        );
        ribosome::run_dna(
            wasm.clone(),
            Some(args_bytes),
            WasmCallData::new_zome_call(context, dna_name.to_string(), zome_call),
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
    ) -> (JsonString, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(canonical_name);
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        let dna_name = &dna.name.to_string().clone();
        let (instance, context) =
            test_instance_and_context(dna, None).expect("Could not create test instance");

        let call_result =
            test_zome_api_function_call(&dna_name, context.clone(), &instance, &wasm, args_bytes);
        (call_result, context)
    }
}
