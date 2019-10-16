extern crate hdk;
extern crate holochain_conductor_lib;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate holochain_wasm_utils;
extern crate tempfile;
extern crate test_utils;

use hdk::error::{ZomeApiError, ZomeApiResult};

use holochain_core_types::{
    crud_status::CrudStatus,
    entry::Entry,
    error::{HolochainError, RibosomeEncodedValue, RibosomeEncodingBits},
};
use holochain_json_api::json::JsonString;

use holochain_core_types::error::CoreError;
use holochain_persistence_api::{cas::content::Address, hash::HashString};

use holochain_wasm_utils::api_serialization::get_links::{GetLinksResult, LinksResult};

use test_utils::{
    assert_zome_internal_errors_equivalent, generate_zome_internal_error, make_test_call,
    start_holochain_instance, wait_for_zome_result, TestEntry,
};

use std::{thread, time::Duration};

//
// These empty function definitions below are needed for the windows linker
//
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
pub fn hc_encrypt(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
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
pub fn hc_crypto(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_meta(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_sign_one_time(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
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
pub fn hc_get_links_count(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
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
pub fn __list_traits(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn __list_functions(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_remove_link(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_list(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_new_random(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_derive_seed(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_derive_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_sign(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_keystore_get_public_key(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_capability_grant(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_commit_capability_claim(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub fn hc_emit_signal(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[test]
pub fn test_invalid_target_link() {
    let (mut hc, _, _signal_receiver) =
        start_holochain_instance("test_invalid_target_link", "alice");
    let result = make_test_call(
        &mut hc,
        "link_tag_validation",
        r#"{"stuff1" : "first","stuff2":"second","tag":"muffins"}"#,
    );
    let expected_result: ZomeApiResult<()> =
        serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    let zome_internal_error =
        generate_zome_internal_error(String::from(r#"{"ValidationFailed":"invalid tag"}"#));
    assert_zome_internal_errors_equivalent(&expected_result.unwrap_err(), &zome_internal_error)
}

#[test]
pub fn test_bad_links() {
    let (mut hc, _, _signal_receiver) = start_holochain_instance("test_bad_links", "alice");
    let result = make_test_call(
        &mut hc,
        "create_and_link_tagged_entry_bad_link",
        r#"{"content" : "message","tag":"maiffins"}"#,
    );

    let expected_result: ZomeApiResult<()> =
        serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    let zome_internal_error = generate_zome_internal_error(String::from(
        r#"{"ErrorGeneric":"Base for link not found"}"#,
    ));
    assert_zome_internal_errors_equivalent(&expected_result.unwrap_err(), &zome_internal_error);
}

#[test]
pub fn test_links_with_immediate_timeout() {
    let (mut hc, _, _signal_receiver) =
        start_holochain_instance("test_links_with_immediate_timeout", "alice");
    make_test_call(
        &mut hc,
        "create_and_link_tagged_entry",
        r#"{"content": "message me","tag":"tag me"}"#,
    )
    .expect("Could not call make call method");

    let result = make_test_call(&mut hc, "my_entries_immediate_timeout", r#"{}"#);
    let expected_result: ZomeApiResult<()> =
        serde_json::from_str::<ZomeApiResult<()>>(&result.clone().unwrap().to_string()).unwrap();
    let zome_internal_error = generate_zome_internal_error(String::from(r#""Timeout""#));;
    assert_zome_internal_errors_equivalent(&expected_result.unwrap_err(), &zome_internal_error);
}

#[test]
pub fn test_links_with_load() {
    let (mut hc, _, _signal_receiver) = start_holochain_instance("test_links_with_load", "alice");
    let result = make_test_call(
        &mut hc,
        "create_and_link_tagged_entry",
        r#"{"content": "message me","tag":"tag me"}"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);

    let expected_result = wait_for_zome_result::<Vec<TestEntry>>(
        &mut hc,
        "my_entries_with_load",
        r#"{}"#,
        |cond| cond.len() == 1,
        12,
    );
    println!("got first links");
    let expected_links = expected_result.expect("Could not get links for test");
    assert_eq!(expected_links[0].stuff, "message me".to_string());

    let result = make_test_call(
        &mut hc,
        "delete_link_tagged_entry",
        r#"{"content": "message me","tag":"tag me"}"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);

    //query for deleted links
    let expected_result = wait_for_zome_result::<GetLinksResult>(
        &mut hc,
        "get_my_entries_by_tag",
        r#"{"tag" : "tag me","status":"Deleted"}"#,
        |cond| cond.links().len() == 1,
        12,
    );
    let expected_links = expected_result.unwrap().clone();
    assert_eq!(expected_links.links().len(), 1);

    //try get links and load with nothing, not sure of necessary more of a type system check
    let expected_result = wait_for_zome_result::<Vec<TestEntry>>(
        &mut hc,
        "my_entries_with_load",
        r#"{}"#,
        |cond| cond.len() == 0,
        12,
    );
    let expected_links = expected_result.unwrap().clone();

    assert_eq!(expected_links.len(), 0);
}

#[test]
fn can_validate_links() {
    let (mut hc, _, _) = start_holochain_instance("can_validate_links", "alice");
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
fn create_tag_and_retrieve() {
    let (mut hc, _, _signal_receiver) =
        start_holochain_instance("create_tag_and_retrieve", "alice");
    let result = make_test_call(
        &mut hc,
        "create_and_link_tagged_entry",
        r#"{"content": "message me","tag":"tag me"}"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);

    let result = make_test_call(
        &mut hc,
        "create_and_link_tagged_entry",
        r#"{"content": "message me once","tag":"tag another me"}"#,
    );
    assert!(result.is_ok(), "result = {:?}", result);

    let expected_result = wait_for_zome_result::<GetLinksResult>(
        &mut hc,
        "get_my_entries_by_tag",
        r#"{"tag" : "tag another me"}"#,
        |cond| cond.links().len() == 1,
        6,
    );
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links
        .links()
        .iter()
        .any(|s| s.tag == "tag another me"));
    assert!(expected_links
        .links()
        .iter()
        .any(|s| s.address == HashString::from("QmeuyJUoXHnU9GJT2LxnnNMmjDbvq1GGsa99pjmo1gPo4Y")));

    let expected_result = wait_for_zome_result::<GetLinksResult>(
        &mut hc,
        "get_my_entries_by_tag",
        r#"{"tag" : "tag me"}"#,
        |cond| cond.links().len() == 1,
        6,
    );
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links.links().iter().any(|s| s.tag == "tag me"));
    assert!(expected_links
        .links()
        .iter()
        .any(|s| s.address == HashString::from("QmPdCLGkzp9daTcwbKePno9SySameXGRqdM4TfTGkju6Mo")));

    let expected_result = wait_for_zome_result::<GetLinksResult>(
        &mut hc,
        "get_my_entries_by_tag",
        r#"{}"#,
        |cond| cond.links().len() == 2,
        6,
    );
    let expected_links = expected_result.unwrap().clone();
    assert!(expected_links
        .links()
        .iter()
        .any(|s| s.tag == "tag another me"));
    assert!(expected_links.links().iter().any(|s| s.tag == "tag me"));
}

#[test]
fn can_link_entries() {
    let (mut hc, _, _) = start_holochain_instance("can_link_entries", "alice");

    let result = make_test_call(&mut hc, "link_two_entries", r#"{}"#);
    assert!(result.is_ok(), "\t result = {:?}", result);
}

#[test]
#[cfg(test)]
fn can_roundtrip_links() {
    let (mut hc, _, _) = start_holochain_instance("can_roundtrip_links", "alice");
    // Create links
    let result = make_test_call(&mut hc, "links_roundtrip_create", r#"{}"#);
    let maybe_address: Result<Address, String> =
        serde_json::from_str(&String::from(result.unwrap())).unwrap();
    let entry_address = maybe_address.unwrap();

    // expected results
    let entry_2 = Entry::App(
        "testEntryType".into(),
        TestEntry {
            stuff: "entry2".into(),
        }
        .into(),
    );
    let entry_3 = Entry::App(
        "testEntryType".into(),
        TestEntry {
            stuff: "entry3".into(),
        }
        .into(),
    );
    let entry_address_2 = Address::from("QmdQVqSuqbrEJWC8Va85PSwrcPfAB3EpG5h83C3Vrj62hN");
    let entry_address_3 = Address::from("QmPn1oj8ANGtxS5sCGdKBdSBN63Bb6yBkmWrLc9wFRYPtJ");

    let expected_links: Result<GetLinksResult, HolochainError> = Ok(GetLinksResult::new(vec![
        LinksResult {
            address: entry_address_2.clone(),
            headers: Vec::new(),
            tag: "test-tag".into(),
            status: CrudStatus::Live,
        },
        LinksResult {
            address: entry_address_3.clone(),
            headers: Vec::new(),
            tag: "test-tag".into(),
            status: CrudStatus::Live,
        },
    ]));
    let expected_links = JsonString::from(expected_links);

    let expected_entries: ZomeApiResult<Vec<ZomeApiResult<Entry>>> =
        Ok(vec![Ok(entry_2.clone()), Ok(entry_3.clone())]);

    let expected_links_reversed: Result<GetLinksResult, HolochainError> =
        Ok(GetLinksResult::new(vec![
            LinksResult {
                address: entry_address_3.clone(),
                headers: Vec::new(),
                tag: "test-tag".into(),
                status: CrudStatus::Live,
            },
            LinksResult {
                address: entry_address_2.clone(),
                headers: Vec::new(),
                tag: "test-tag".into(),
                status: CrudStatus::Live,
            },
        ]));
    let expected_links_reversed = JsonString::from(expected_links_reversed);

    let expected_entries_reversed: ZomeApiResult<Vec<ZomeApiResult<Entry>>> =
        Ok(vec![Ok(entry_3.clone()), Ok(entry_2.clone())]);

    // Polling loop because the links have to get pushed over the in-memory network and then validated
    // which includes requesting a validation package and receiving it over the in-memory network.
    // All of that happens asynchronously and takes longer depending on computing resources
    // (i.e. longer on a slow CI and when multiple tests are run simultaneausly).
    let mut both_links_present = false;
    let mut tries = 0;
    let mut result_of_get = JsonString::from_json("{}");
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
