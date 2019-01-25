use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::get_entry_result::get_entry_result_workflow,
};
use holochain_wasm_utils::api_serialization::get_entry::GetEntryArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetAppEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetEntryArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetEntryArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            runtime.context.log(format!(
                "err/zome: invoke_get_entry() failed to deserialize: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    // Create workflow future and block on it
    let result = runtime.context.block_on(get_entry_result_workflow(&runtime.context, &input));
    // Store result in wasm memory
    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use self::wabt::Wat2Wasm;
    use crate::{
        instance::tests::{test_context_and_logger, test_instance},
        nucleus::{
            ribosome::{
                self,
                api::{
                    commit::tests::test_commit_args_bytes,
                    tests::{test_parameters, test_zome_name},
                },
            },
            tests::{test_capability_call, test_capability_name},
            ZomeFnCall,
        },
    };
    use holochain_core_types::{
        cas::content::{Address, AddressableContent},
        crud_status::CrudStatus,
        entry::{test_entry, EntryWithMeta},
        error::ZomeApiInternalResult,
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::get_entry::*;
    use std::sync::Arc;

    /// dummy get args from standard test entry
    pub fn test_get_args_bytes() -> Vec<u8> {
        let entry_args = GetEntryArgs {
            address: test_entry().address(),
            options: GetEntryOptions::new(
                StatusRequestKind::Latest,
                true,
                false,
                false,
                Default::default(),
            ),
        };
        JsonString::from(entry_args).into_bytes()
    }

    /// dummy get args from standard test entry
    pub fn test_get_args_unknown() -> Vec<u8> {
        let entry_args = GetEntryArgs {
            address: Address::from("xxxxxxxxx"),
            options: GetEntryOptions::new(
                StatusRequestKind::Latest,
                true,
                false,
                false,
                Default::default(),
            ),
        };
        JsonString::from(entry_args).into_bytes()
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

    (func
        (export "__list_functions")
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
        let netname = Some("test_get_round_trip");
        let wasm = test_get_round_trip_wat();
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability_name(),
            wasm.clone(),
        );
        let instance =
            test_instance(dna.clone(), netname).expect("Could not initialize test instance");
        let (context, _) = test_context_and_logger("joan", netname);
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
            Some(test_capability_call()),
            "commit_dispatch",
            test_parameters(),
        );
        let call_result = ribosome::run_dna(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &commit_call,
            Some(test_commit_args_bytes()),
        )
        .expect("test should be callable");

        assert_eq!(
            call_result,
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(
                    test_entry().address()
                ))) + "\u{0}"
            ),
        );

        let get_call = ZomeFnCall::new(
            &test_zome_name(),
            Some(test_capability_call()),
            "get_dispatch",
            test_parameters(),
        );
        let call_result = ribosome::run_dna(
            &dna.name.to_string(),
            Arc::clone(&context),
            wasm.clone(),
            &get_call,
            Some(test_get_args_bytes()),
        )
        .expect("test should be callable");

        let entry_result = GetEntryResult::new(
            StatusRequestKind::Latest,
            Some(&EntryWithMeta {
                entry: test_entry(),
                crud_status: CrudStatus::Live,
                maybe_crud_link: None,
            }),
        );
        assert_eq!(
            JsonString::from(String::from(JsonString::from(
                ZomeApiInternalResult::success(entry_result)
            ))),
            call_result,
        );
    }

    #[test]
    #[cfg(not(windows))]
    /// test that we get status NotFound on an obviously broken address
    fn test_get_not_found() {
        // let wasm = test_get_round_trip_wat();
        // let dna = test_utils::create_test_dna_with_wasm(
        //     &test_zome_name(),
        //     &test_capability_name(),
        //     wasm.clone(),
        // );
        // let instance = test_instance(dna.clone()).expect("Could not initialize test instance");
        // let (context, _) = test_context_and_logger("joan");
        // let context = instance.initialize_context(context);
        //
        // println!("{:?}", instance.state().agent().top_chain_header());
        // println!(
        //     "{:?}",
        //     instance
        //         .state()
        //         .agent()
        //         .top_chain_header()
        //         .expect("top chain_header was None")
        //         .address()
        // );
        //
        // let get_call = ZomeFnCall::new(
        //     &test_zome_name(),
        //     Some(test_capability_call()),
        //     "get_dispatch",
        //     test_parameters(),
        // );
        // let call_result = ribosome::run_dna(
        //     &dna.name.to_string(),
        //     Arc::clone(&context),
        //     wasm.clone(),
        //     &get_call,
        //     Some(test_get_args_unknown()),
        // )
        // .expect("test should be callable");
        //
        // assert_eq!(
        //     JsonString::from(String::from(JsonString::from(
        //         ZomeApiInternalResult::success(GetEntryResult::new(StatusRequestKind::Latest, None))
        //     ))),
        //     call_result,
        // );
    }

}
