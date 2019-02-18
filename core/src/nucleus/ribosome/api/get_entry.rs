use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::get_entry_result::get_entry_result_workflow,
};
use holochain_wasm_utils::api_serialization::get_entry::GetEntryArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetAppEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetEntryArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_get_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    let context = zome_call_data.context;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetEntryArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        // Exit on error
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_get_entry() failed to deserialize: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    // Create workflow future and block on it
    let result = context.block_on(get_entry_result_workflow(&context, &input));
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
                runtime::WasmCallData,
            },
            tests::test_capability_call,
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
            (param i64)
            (result i64)
        )
    )

    (import "env" "hc_commit_entry"
        (func $commit
            (param i64)
            (result i64)
        )
    )

    (memory 1)
    (export "memory" (memory 0))

    (func
        (export "get_dispatch")
            (param $allocation i64)
            (result i64)

        (call
            $get
            (get_local $allocation)
        )
    )

    (func
        (export "commit_dispatch")
            (param $allocation i64)
            (result i64)

        (call
            $commit
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
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());
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
            test_capability_call(context.clone(), "commit_dispatch", test_parameters()),
            "commit_dispatch",
            test_parameters(),
        );
        let call_result = ribosome::run_dna(
            wasm.clone(),
            Some(test_commit_args_bytes()),
            WasmCallData::new_zome_call(Arc::clone(&context), dna.name.clone(), commit_call),
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
            test_capability_call(context.clone(), "get_dispatch", test_parameters()),
            "get_dispatch",
            test_parameters(),
        );
        let call_result = ribosome::run_dna(
            wasm.clone(),
            Some(test_get_args_bytes()),
            WasmCallData::new_zome_call(Arc::clone(&context), dna.name, get_call),
        )
        .expect("test should be callable");

        let entry = test_entry();
        let entry_with_meta = EntryWithMeta {
            entry: entry.clone(),
            crud_status: CrudStatus::Live,
            maybe_crud_link: None,
        };
        // let header = create_new_chain_header(&entry, context.clone(), &None);
        let entry_result =
            GetEntryResult::new(StatusRequestKind::Latest, Some((&entry_with_meta, vec![])));
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
        //     test_capability_call(),
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
