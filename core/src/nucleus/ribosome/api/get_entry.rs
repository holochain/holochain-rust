use futures::executor::block_on;
use holochain_wasm_utils::api_serialization::get_entry::{GetEntryArgs, GetEntryResult};
use nucleus::{actions::get_entry::get_entry, ribosome::Runtime};
use serde_json;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

/// ZomeApiFunction::GetAppEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetEntryArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_entry(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let res_entry: Result<GetEntryArgs, _> = serde_json::from_str(&args_str);
    // Exit on error
    if res_entry.is_err() {
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let input = res_entry.unwrap();

    let future = get_entry(&runtime.context, input.address);
    let result = block_on(future);
    match result {
        Err(_) => ribosome_error_code!(Unspecified),
        Ok(maybe_entry) => match maybe_entry {
            Some(entry) => {
                let result = GetEntryResult::found(entry.to_string());
                let result_string =
                    serde_json::to_string(&result).expect("Could not serialize GetAppEntryResult");
                runtime.store_utf8(&result_string)
            }
            None => {
                let result = GetEntryResult::not_found();
                let result_string =
                    serde_json::to_string(&result).expect("Could not serialize GetAppEntryResult");
                runtime.store_utf8(&result_string)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    extern crate test_utils;
    extern crate wabt;

    use self::wabt::Wat2Wasm;
    use super::GetEntryArgs;
    use holochain_core_types::{
        cas::content::AddressableContent, entry::test_entry, hash::HashString,
    };
    use instance::tests::{test_context_and_logger, test_instance};
    use nucleus::{
        ribosome::{
            self,
            api::{
                commit::tests::test_commit_args_bytes,
                tests::{test_capability, test_parameters, test_zome_name},
            },
        },
        ZomeFnCall,
    };
    use serde_json;
    use std::sync::Arc;

    /// dummy get args from standard test entry
    pub fn test_get_args_bytes() -> Vec<u8> {
        let args = GetEntryArgs {
            address: test_entry().address().into(),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    /// dummy get args from standard test entry
    pub fn test_get_args_unknown() -> Vec<u8> {
        let args = GetEntryArgs {
            address: HashString::from(String::from("xxxxxxxxx")),
        };
        serde_json::to_string(&args).unwrap().into_bytes()
    }

    /// wat string that exports both get and a commit dispatches so we can test a round trip
    pub fn test_get_round_trip_wat() -> Vec<u8> {
        Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                r#"
(module
    (import "env" "hc_get_entry"
        (func $get
            (param i32)
            (result i32)
        )
    )

    (import "env" "hc_commit_entry"
        (func $commit
            (param i32)
            (result i32)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "get_dispatch")
            (param $allocation i32)
            (result i32)

        (call
            $get
            (get_local $allocation)
        )
    )

    (func
        (export "commit_dispatch")
            (param $allocation i32)
            (result i32)

        (call
            $commit
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
            )
            .unwrap()
            .as_ref()
            .to_vec()
    }

    #[test]
    /// test that we can round trip bytes through a get action and it comes back from wasm
    fn test_get_round_trip() {
        let wasm = test_get_round_trip_wat();
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );
        let instance = test_instance(dna.clone()).expect("Could not initialize test instance");
        let (context, _) = test_context_and_logger("joan");
        let context = instance.initialize_context(context);

        println!("{:?}", instance.state().agent().top_chain_header());
        println!(
            "{:?}",
            instance
                .state()
                .agent()
                .top_chain_header()
                .expect("top chain_header was None")
                .address()
        );

        let commit_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            "commit_dispatch",
            &test_parameters(),
        );
        let call_result = ribosome::run_dna(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &commit_call,
            Some(test_commit_args_bytes()),
        ).expect("test should be callable");

        assert_eq!(
            call_result,
            format!(
                r#"{{"address":"{}","validation_failure":""}}"#,
                test_entry().address()
            ) + "\u{0}",
        );

        let get_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            "get_dispatch",
            &test_parameters(),
        );
        let call_result = ribosome::run_dna(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &get_call,
            Some(test_get_args_bytes()),
        ).expect("test should be callable");

        let mut expected = "".to_owned();
        expected.push_str("{\"status\":\"Found\",\"entry\":\"test entry value\"}\u{0}");

        assert_eq!(expected, call_result);
    }

    #[test]
    /// test that we get status NotFound on an obviously broken hash
    fn test_get_not_found() {
        let wasm = test_get_round_trip_wat();
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );
        let instance = test_instance(dna.clone()).expect("Could not initialize test instance");
        let (context, _) = test_context_and_logger("joan");
        let context = instance.initialize_context(context);

        println!("{:?}", instance.state().agent().top_chain_header());
        println!(
            "{:?}",
            instance
                .state()
                .agent()
                .top_chain_header()
                .expect("top chain_header was None")
                .address()
        );

        let get_call = ZomeFnCall::new(
            &test_zome_name(),
            &test_capability(),
            "get_dispatch",
            &test_parameters(),
        );
        let call_result = ribosome::run_dna(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &get_call,
            Some(test_get_args_unknown()),
        ).expect("test should be callable");

        let mut expected = "".to_owned();
        expected.push_str("{\"status\":\"NotFound\",\"entry\":\"\"}\u{0}");

        assert_eq!(expected, call_result);
    }

}
