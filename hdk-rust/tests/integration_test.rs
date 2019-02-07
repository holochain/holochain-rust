#![feature(try_from)]
extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate tempfile;
extern crate test_utils;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate hdk;
extern crate holochain_wasm_utils;
#[macro_use]
extern crate holochain_core_types_derive;

use hdk::error::{ZomeApiError, ZomeApiResult};
use holochain_conductor_api::{error::HolochainResult, *};
use holochain_core::logger::TestLogger;
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::CrudStatus,
    dna::{
        capabilities::{Capability, CapabilityCall, CapabilityType},
        entry_types::{EntryTypeDef, LinksTo},
        fn_declarations::FnDeclaration,
        zome::{ZomeCapabilities, ZomeFnDeclarations},
    },
    entry::{
        entry_type::{test_app_entry_type, EntryType},
        Entry, EntryWithMeta,
    },
    error::{CoreError, HolochainError, RibosomeEncodedValue, RibosomeEncodingBits},
    hash::HashString,
    json::JsonString,
};
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::{GetEntryResult, StatusRequestKind},
        get_links::GetLinksResult,
    },
    wasm_target_dir,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use test_utils::*;

#[no_mangle]
pub fn hc_init_globals(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_get_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_entry_address(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_query(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_update_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_remove_entry(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_send(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_property(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_debug(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_call(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_verify_signature(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_link_entries(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_get_links(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_start_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_close_bundle(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_sleep(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn zome_setup(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn __list_capabilities(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}
#[no_mangle]
pub fn __list_functions(_: u32) -> u32 {
    0
}

#[no_mangle]
pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

pub fn create_test_defs_with_fn_names(
    fn_names: Vec<&str>,
) -> (ZomeFnDeclarations, ZomeCapabilities) {
    let mut capability = Capability::new(CapabilityType::Public);
    let mut fn_declarations = Vec::new();

    for fn_name in fn_names {
        capability.functions.push(String::from(fn_name));
        let mut fn_decl = FnDeclaration::new();
        fn_decl.name = String::from(fn_name);
        fn_declarations.push(fn_decl);
    }
    let mut capabilities = BTreeMap::new();
    capabilities.insert("test_cap".to_string(), capability);
    (fn_declarations, capabilities)
}

#[derive(Deserialize, Serialize, Default, Debug, DefaultJson)]
/// dupes wasm_test::EntryStruct;
struct EntryStruct {
    stuff: String,
}

fn example_valid_entry() -> Entry {
    Entry::App(
        test_app_entry_type().into(),
        EntryStruct {
            stuff: "non fail".into(),
        }
        .into(),
    )
}

fn example_valid_entry_result() -> GetEntryResult {
    let entry = example_valid_entry();
    let entry_with_meta = &EntryWithMeta {
        entry: entry.clone(),
        crud_status: CrudStatus::Live,
        maybe_crud_link: None,
    };
    GetEntryResult::new(StatusRequestKind::Latest, Some((entry_with_meta, vec![])))
}

fn example_valid_entry_params() -> String {
    format!(
        "{{\"entry\":{}}}",
        String::from(JsonString::from(example_valid_entry())),
    )
}

fn example_valid_entry_address() -> Address {
    Address::from("QmefcRdCAXM2kbgLW2pMzqWhUvKSDvwfFSVkvmwKvBQBHd")
}

fn start_holochain_instance<T: Into<String>>(
    uuid: T,
    agent_name: T,
) -> (Holochain, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(&format!(
        "{}/wasm32-unknown-unknown/release/test_globals.wasm",
        wasm_target_dir("hdk-rust/", "wasm-test/"),
    ));
    let defs = create_test_defs_with_fn_names(vec![
        "check_global",
        "check_commit_entry",
        "check_commit_entry_macro",
        "check_get_entry_result",
        "check_get_entry",
        "send_tweet",
        "commit_validation_package_tester",
        "link_two_entries",
        "links_roundtrip_create",
        "links_roundtrip_get",
        "links_roundtrip_get_and_load",
        "link_validation",
        "check_query",
        "check_app_entry_address",
        "check_sys_entry_address",
        "check_call",
        "check_call_with_args",
        "update_entry_ok",
        "remove_entry_ok",
        "remove_modified_entry_ok",
        "send_message",
        "sleep",
        "remove_link",
    ]);
    let mut dna = create_test_dna_with_defs("test_zome", defs, &wasm);
    dna.uuid = uuid.into();

    // TODO: construct test DNA using the auto-generated JSON feature
    // The code below is fragile!
    // We have to manually construct a Dna struct that reflects what we defined using define_zome!
    // in wasm-test/src/lib.rs.
    // In a production setting, hc would read the auto-generated JSON to make sure the Dna struct
    // matches up. We should do the same in test.
    {
        let entry_types = &mut dna.zomes.get_mut("test_zome").unwrap().entry_types;
        entry_types.insert(
            EntryType::from("validation_package_tester"),
            EntryTypeDef::new(),
        );

        let test_entry_type = &mut entry_types
            .get_mut(&EntryType::from("testEntryType"))
            .unwrap();
        test_entry_type.links_to.push(LinksTo {
            target_type: String::from("testEntryType"),
            tag: String::from("test-tag"),
        });
    }

    {
        let entry_types = &mut dna.zomes.get_mut("test_zome").unwrap().entry_types;
        let mut link_validator = EntryTypeDef::new();
        link_validator.links_to.push(LinksTo {
            target_type: String::from("link_validator"),
            tag: String::from("longer"),
        });
        entry_types.insert(EntryType::from("link_validator"), link_validator);
    }

    let (context, test_logger) =
        test_context_and_logger_with_network_name(&agent_name.into(), Some(&dna.uuid));
    let mut hc =
        Holochain::new(dna.clone(), context).expect("could not create new Holochain instance.");

    // Run the holochain instance
    hc.start().expect("couldn't start");
    (hc, test_logger)
}

fn make_test_call(hc: &mut Holochain, fn_name: &str, params: &str) -> HolochainResult<JsonString> {
    hc.call(
        "test_zome",
        Some(CapabilityCall::new(Address::from("test_token"), None)),
        fn_name,
        params,
    )
}

#[test]
fn can_use_globals() {
    let (mut hc, _) = start_holochain_instance("can_use_globals", "alice");
    // Call the exposed wasm function that calls the debug API function for printing all GLOBALS
    let result = make_test_call(&mut hc, "check_global", r#"{}"#);
    assert_eq!(
        result.clone(),
        Ok(JsonString::from(HashString::from(
            "alice-----------------------------------------------------------------------------AAAIuDJb4M"
        ))),
        "result = {:?}",
        result
    );
}

#[test]
fn can_commit_entry() {
    let (mut hc, _) = start_holochain_instance("can_commit_entry", "alice");

    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry",
        &String::from(JsonString::from(example_valid_entry())),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(example_valid_entry_address()),
    );
}
#[test]
fn can_commit_entry_macro() {
    let (mut hc, _) = start_holochain_instance("can_commit_entry_macro", "alice");
    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    let expected: ZomeApiResult<Address> = Ok(Address::from(
        "QmefcRdCAXM2kbgLW2pMzqWhUvKSDvwfFSVkvmwKvBQBHd",
    ));
    assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_round_trip() {
    let (mut hc, test_logger) = start_holochain_instance("can_round_trip", "alice");
    let result = make_test_call(
        &mut hc,
        "send_tweet",
        r#"{ "author": "bob", "content": "had a boring day" }"#,
    );
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"first\":\"bob\",\"second\":\"had a boring day\"}"),
    );

    let test_logger = test_logger.lock().unwrap();

    println!("{:?}", *test_logger);
}

#[test]
#[cfg(not(windows))]
fn can_get_entry_ok() {
    let (mut hc, _) = start_holochain_instance("can_get_entry_ok", "alice");
    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    let expected: ZomeApiResult<Address> = Ok(example_valid_entry_address());
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected));

    let result = make_test_call(
        &mut hc,
        "check_get_entry_result",
        &String::from(JsonString::from(json!({
            "entry_address": example_valid_entry_address()
        }))),
    );
    let expected: ZomeApiResult<GetEntryResult> = Ok(example_valid_entry_result());
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected));

    let result = make_test_call(
        &mut hc,
        "check_get_entry",
        &String::from(JsonString::from(json!({
            "entry_address": example_valid_entry_address()
        }))),
    );
    let expected: ZomeApiResult<Entry> = Ok(example_valid_entry());
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected));
}

#[test]
#[cfg(not(windows))]
fn can_get_entry_bad() {
    let (mut hc, _) = start_holochain_instance("can_get_entry_bad", "alice");
    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    let expected: ZomeApiResult<Address> = Ok(example_valid_entry_address());
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected),);
    // test the case with a bad address
    let result = make_test_call(
        &mut hc,
        "check_get_entry_result",
        &String::from(JsonString::from(json!(
            {"entry_address": Address::from("QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx")}
        ))),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    let empty_entry_result = GetEntryResult::new(StatusRequestKind::Latest, None);
    let expected: ZomeApiResult<GetEntryResult> = Ok(empty_entry_result);
    assert_eq!(result.unwrap(), JsonString::from(expected));

    // test the case with a bad address
    let result = make_test_call(
        &mut hc,
        "check_get_entry",
        &String::from(JsonString::from(json!(
            {"entry_address": Address::from("QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx")}
        ))),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    let expected: ZomeApiResult<Option<Entry>> = Ok(None);
    assert_eq!(result.unwrap(), JsonString::from(expected));
}

#[test]
#[cfg(not(windows))] // TODO does not work on windows because of different seperator
fn can_invalidate_invalid_commit() {
    let (mut hc, _) = start_holochain_instance("can_invalidate_invalid_commit", "alice");
    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &json!({"entry":
            Entry::App(
                test_app_entry_type().into(),
                EntryStruct {
                    stuff: "FAIL".into(),
                }.into(),
            )
        })
        .to_string(),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"Err\":{\"Internal\":\"{\\\"kind\\\":{\\\"ValidationFailed\\\":\\\"FAIL content is not allowed\\\"},\\\"file\\\":\\\"core/src/nucleus/ribosome/runtime.rs\\\",\\\"line\\\":\\\"94\\\"}\"}}"),
    );
}

#[test]
fn has_populated_validation_data() {
    let (mut hc, _) = start_holochain_instance("has_populated_validation_data", "alice");

    //
    // Add two entries to chain to have something to check ValidationData on
    //
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);

    let expected: ZomeApiResult<Address> = Ok(example_valid_entry_address());
    assert_eq!(result.unwrap(), JsonString::from(expected),);

    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);

    let expected: ZomeApiResult<Address> = Ok(example_valid_entry_address());
    assert_eq!(result.unwrap(), JsonString::from(expected),);

    //
    // Expect the commit in this zome function to fail with a serialized ValidationData struct
    //
    let result = make_test_call(&mut hc, "commit_validation_package_tester", r#"{}"#);

    assert!(result.is_ok(), "\t result = {:?}", result);

    //
    // Deactivating this test for now since ordering of contents change non-deterministically
    //
    /*
    assert_eq!(
        JsonString::from("{\"Err\":{\"Internal\":\"{\\\"package\\\":{\\\"chain_header\\\":{\\\"entry_type\\\":{\\\"App\\\":\\\"validation_package_tester\\\"},\\\"entry_address\\\":\\\"QmYQPp1fExXdKfmcmYTbkw88HnCr3DzMSFUZ4ncEd9iGBY\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmSQqKHPpYZbafF7PXPKx31UwAbNAmPVuSHHxcBoDcYsci\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},\\\"source_chain_entries\\\":[{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"alex\\\",\\\"entry_type\\\":\\\"%agent_id\\\"}],\\\"source_chain_headers\\\":[{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"link_same_type\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRYerwRRXYxmYoxq1LTZMVVRfjNMAeqmdELTNDxURtHEZ\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":\\\"AgentId\\\",\\\"entry_address\\\":\\\"QmQw3V41bAWkQA9kwpNfU3ZDNzr9YW4p9RV4QHhFD3BkqA\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmQJxUSfJe2QoxTyEwKQX9ypbkcNv3cw1vasGTx1CUpJFm\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"}],\\\"custom\\\":null},\\\"sources\\\":[\\\"<insert your agent key here>\\\"],\\\"lifecycle\\\":\\\"Chain\\\",\\\"action\\\":\\\"Commit\\\"}\"}}"),
        result.unwrap(),
    );
    */
}

#[test]
fn can_link_entries() {
    let (mut hc, _) = start_holochain_instance("can_link_entries", "alice");

    let result = make_test_call(&mut hc, "link_two_entries", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(r#"{"Ok":null}"#));
}

#[test]
fn can_remove_link() {
    let (mut hc, _) = start_holochain_instance("can_link_entries", "alice");

    let result = make_test_call(&mut hc, "link_two_entries", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(r#"{"Ok":null}"#));
}
#[test]
#[cfg(not(windows))]
fn can_roundtrip_links() {
    let (mut hc, _) = start_holochain_instance("can_roundtrip_links", "alice");
    // Create links
    let result = make_test_call(&mut hc, "links_roundtrip_create", r#"{}"#);
    let maybe_address: Result<Address, String> =
        serde_json::from_str(&String::from(result.unwrap())).unwrap();
    let entry_address = maybe_address.unwrap();

    // expected results
    let entry_2 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry2".into(),
        }
        .into(),
    );
    let entry_3 = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: "entry3".into(),
        }
        .into(),
    );
    let entry_address_2 = Address::from("QmdQVqSuqbrEJWC8Va85PSwrcPfAB3EpG5h83C3Vrj62hN");
    let entry_address_3 = Address::from("QmPn1oj8ANGtxS5sCGdKBdSBN63Bb6yBkmWrLc9wFRYPtJ");

    let expected_links: Result<GetLinksResult, HolochainError> = Ok(GetLinksResult::new(vec![
        entry_address_2.clone(),
        entry_address_3.clone(),
    ]));
    let expected_links = JsonString::from(expected_links);

    let expected_entries: ZomeApiResult<Vec<ZomeApiResult<Entry>>> =
        Ok(vec![Ok(entry_2.clone()), Ok(entry_3.clone())]);

    let expected_links_reversed: Result<GetLinksResult, HolochainError> =
        Ok(GetLinksResult::new(vec![entry_address_3, entry_address_2]));
    let expected_links_reversed = JsonString::from(expected_links_reversed);

    let expected_entries_reversed: ZomeApiResult<Vec<ZomeApiResult<Entry>>> =
        Ok(vec![Ok(entry_3.clone()), Ok(entry_2.clone())]);

    // Polling loop because the links have to get pushed over the in-memory network and then validated
    // which includes requesting a validation package and receiving it over the in-memory network.
    // All of that happens asynchronously and takes longer depending on computing resources
    // (i.e. longer on a slow CI and when multiple tests are run simultaneausly).
    let mut both_links_present = false;
    let mut tries = 0;
    let mut result_of_get = JsonString::from("");
    while !both_links_present && tries < 10 {
        tries = tries + 1;
        // Now get_links on the base and expect both to be there
        let maybe_result_of_get = make_test_call(
            &mut hc,
            "links_roundtrip_get",
            &format!(r#"{{"address": "{}"}}"#, entry_address),
        );
        let maybe_result_of_load = make_test_call(
            &mut hc,
            "links_roundtrip_get_and_load",
            &format!(r#"{{"address": "{}"}}"#, entry_address),
        );

        assert!(
            maybe_result_of_get.is_ok(),
            "maybe_result_of_get = {:?}",
            maybe_result_of_get
        );
        assert!(
            maybe_result_of_load.is_ok(),
            "maybe_result_of_load = {:?}",
            maybe_result_of_load
        );

        result_of_get = maybe_result_of_get.unwrap();
        let result_of_load = maybe_result_of_load.unwrap();

        println!(
            "can_roundtrip_links: result_of_load - try {}:\n {:?}\n expecting:\n {:?}",
            tries, result_of_load, &expected_entries,
        );

        let ordering1: bool = result_of_get == expected_links;
        let entries_ordering1: bool = result_of_load == JsonString::from(expected_entries.clone());

        let ordering2: bool = result_of_get == expected_links_reversed;
        let entries_ordering2: bool =
            result_of_load == JsonString::from(expected_entries_reversed.clone());

        both_links_present = (ordering1 || ordering2) && (entries_ordering1 || entries_ordering2);
        if !both_links_present {
            // Wait for links to be validated and propagated
            thread::sleep(Duration::from_millis(500));
        }
    }

    assert!(both_links_present, "result = {:?}", result_of_get);
}

#[test]
#[cfg(not(windows))]
fn can_validate_links() {
    let (mut hc, _) = start_holochain_instance("can_validate_links", "alice");
    let params_ok = r#"{"stuff1": "a", "stuff2": "aa"}"#;
    let result = make_test_call(&mut hc, "link_validation", params_ok);
    assert!(result.is_ok(), "result = {:?}", result);

    let params_not_ok = r#"{"stuff1": "aaa", "stuff2": "aa"}"#;
    let result = make_test_call(&mut hc, "link_validation", params_not_ok);
    assert!(result.is_ok(), "result = {:?}", result);
    // Yep, the zome call is ok but what we got back should be a ValidationFailed error,
    // wrapped in a CoreError, wrapped in a ZomeApiError, wrapped in a Result,
    // serialized to JSON :D
    let zome_result: Result<(), ZomeApiError> =
        serde_json::from_str(&result.unwrap().to_string()).unwrap();
    assert!(zome_result.is_err());
    if let ZomeApiError::Internal(error) = zome_result.err().unwrap() {
        let core_error: CoreError = serde_json::from_str(&error).unwrap();
        assert_eq!(
            core_error.kind,
            HolochainError::ValidationFailed("Target stuff is not longer".to_string()),
        );
    } else {
        assert!(false);
    }
}

#[test]
fn can_check_query() {
    let (mut hc, _) = start_holochain_instance("can_check_query", "alice");

    let result = make_test_call(
        &mut hc,
        "check_query",
        r#"{ "entry_type_names": ["testEntryType"], "limit": "0" }"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);

    let expected: ZomeApiResult<Vec<Address>> = Ok(vec![Address::from(
        "QmPn1oj8ANGtxS5sCGdKBdSBN63Bb6yBkmWrLc9wFRYPtJ",
    )]);

    assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_check_app_entry_address() {
    let (mut hc, _) = start_holochain_instance("can_check_app_entry_address", "alice");

    let result = make_test_call(&mut hc, "check_app_entry_address", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);

    let expected: ZomeApiResult<Address> = Ok(Address::from(
        "QmSbNw63sRS4VEmuqFBd7kJT6V9pkEpMRMY2LWvjNAqPcJ",
    ));
    assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_check_sys_entry_address() {
    let (mut hc, _) = start_holochain_instance("can_check_sys_entry_address", "alice");

    let _result = make_test_call(&mut hc, "check_sys_entry_address", r#"{}"#);
    // TODO
    //    assert!(result.is_ok(), "result = {:?}", result);
    //    assert_eq!(
    //        result.unwrap(),
    //        r#"{"result":"QmYmZyvDda3ygMhNnEjx8p9Q1TonHG9xhpn9drCptRT966"}"#,
    //    );
}

#[test]
fn can_check_call() {
    //let (mut hc, _) = start_holochain_instance("can_check_call", "alice");

    //let result = make_test_call(&mut hc, "check_call", r#"{}"#);
    //assert!(result.is_ok(), "result = {:?}", result);

    //let inner_expected: ZomeApiResult<Address> = Ok(Address::from(
    //    "QmSbNw63sRS4VEmuqFBd7kJT6V9pkEpMRMY2LWvjNAqPcJ",
    //));
    //let expected: ZomeApiResult<ZomeApiInternalResult> =
    //    Ok(ZomeApiInternalResult::success(inner_expected));

    //assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_check_call_with_args() {
    //let (mut hc, _) = start_holochain_instance("can_check_call_with_args", "alice");

    //let result =make_test_call(&mut hc,
    //    "check_call_with_args",
    //    &String::from(JsonString::empty_object()),
    //);
    //println!("\t result = {:?}", result);
    //assert!(result.is_ok(), "\t result = {:?}", result);

    //let expected_inner: ZomeApiResult<Address> = Ok(Address::from(
    //    "QmefcRdCAXM2kbgLW2pMzqWhUvKSDvwfFSVkvmwKvBQBHd",
    //));
    //let expected: ZomeApiResult<ZomeApiInternalResult> =
    //    Ok(ZomeApiInternalResult::success(expected_inner));

    //assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_remove_entry() {
    let (mut hc, _) = start_holochain_instance("can_remove_entry", "alice");
    let result = make_test_call(&mut hc, "remove_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"items\":[{\"meta\":{\"address\":\"QmefcRdCAXM2kbgLW2pMzqWhUvKSDvwfFSVkvmwKvBQBHd\",\"entry_type\":{\"App\":\"testEntryType\"},\"crud_status\":\"deleted\"},\"entry\":{\"App\":[\"testEntryType\",\"{\\\"stuff\\\":\\\"non fail\\\"}\"]},\"headers\":[]}],\"crud_links\":{\"QmefcRdCAXM2kbgLW2pMzqWhUvKSDvwfFSVkvmwKvBQBHd\":\"QmUhD35RLLvDJ7dGsonTTiHUirckQSbf7ceDC1xWVTrHk6\"}}"
        ),
    );
}

#[test]
fn can_update_entry() {
    let (mut hc, _) = start_holochain_instance("can_update_entry", "alice");
    let result = make_test_call(&mut hc, "update_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}

#[test]
fn can_remove_modified_entry() {
    let (mut hc, _) = start_holochain_instance("can_remove_modified_entry", "alice");
    let result = make_test_call(&mut hc, "remove_modified_entry_ok", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}

#[test]
fn can_send_and_receive() {
    let (mut hc, _) = start_holochain_instance("can_send_and_receive", "alice");
    let result = make_test_call(&mut hc, "check_global", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    let agent_id = result.unwrap().to_string();

    let (mut hc2, _) = start_holochain_instance("can_send_and_receive", "bob");
    let params = format!(r#"{{"to_agent": {}, "message": "TEST"}}"#, agent_id);
    let result = make_test_call(&mut hc2, "send_message", &params);
    assert!(result.is_ok(), "result = {:?}", result);

    let entry_committed_by_receive = Entry::App(
        "testEntryType".into(),
        EntryStruct {
            stuff: String::from("TEST"),
        }
        .into(),
    );

    let address = entry_committed_by_receive.address().to_string();

    let expected: ZomeApiResult<String> = Ok(format!("Committed: 'TEST' / address: {}", address));
    assert_eq!(result.unwrap(), JsonString::from(expected),);

    let result = make_test_call(
        &mut hc,
        "check_get_entry",
        &String::from(JsonString::from(json!({
            "entry_address": address,
        }))),
    );

    let expected: ZomeApiResult<Entry> = Ok(entry_committed_by_receive);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn sleep_smoke_test() {
    let (mut hc, _) = start_holochain_instance("sleep_smoke_test", "alice");
    let result = make_test_call(&mut hc, "sleep", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}
