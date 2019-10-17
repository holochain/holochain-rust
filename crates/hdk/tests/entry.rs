extern crate holochain_conductor_lib;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate tempfile;
extern crate test_utils;
#[macro_use]
extern crate serde_json;
extern crate hdk;
extern crate holochain_wasm_utils;

use hdk::error::ZomeApiResult;

use holochain_core_types::{
    entry::Entry,
    error::{RibosomeEncodedValue, RibosomeEncodingBits},
};

use holochain_json_api::json::JsonString;
use holochain_persistence_api::{
    cas::content::{Address, AddressableContent},
    hash::HashString,
};

use holochain_wasm_utils::api_serialization::get_entry::{GetEntryResult, StatusRequestKind};
use std::path::PathBuf;
use test_utils::{
    empty_string_validation_fail_entry, example_valid_entry, example_valid_entry_address,
    example_valid_entry_params, example_valid_entry_result, make_test_call,
    start_holochain_instance, wait_for_zome_result,
};

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
fn hash_entry() {
    let (mut hc, _, _) = start_holochain_instance("hash_entry", "alice");
    let params = r#"{"content":"this is to hash"}"#;
    let result = make_test_call(&mut hc, "hash_entry", &params);
    assert_eq!(
        result,
        Ok(JsonString::from(
            r#"{"Ok":"QmNsza9FP5Unf45UixMfnPvkg4SY8aYcPjvX8FtMzVfpas"}"#
        )),
        "result = {:?}",
        result,
    );
}

#[test]
pub fn create_and_retrieve_private_entry() {
    let (mut hc, _, _signal_receiver) =
        start_holochain_instance("create_and_retrieve_private_entry", "alice");
    let result = make_test_call(
        &mut hc,
        "create_priv_entry",
        r#"{"content":"check this out"}"#,
    );

    let expected_result: ZomeApiResult<Address> =
        serde_json::from_str::<ZomeApiResult<Address>>(&result.clone().unwrap().to_string())
            .unwrap();
    let zome_call = format!(r#"{{"address":"{}"}}"#, expected_result.unwrap());

    let expected_result = wait_for_zome_result::<Option<Entry>>(
        &mut hc,
        "get_entry",
        &zome_call,
        |maybe_entry| maybe_entry.is_some(),
        6,
    );
    let entry = expected_result.expect("Could not get entry for test");
    assert_eq!(
        entry.unwrap().address(),
        HashString::from("QmYop82eqkWo5f9eLx8dj89ppGGyE11zmEGQy8jMF3nVxp")
    )
}

#[test]
pub fn test_bad_entry() {
    let (mut hc, _, _signal_receiver) = start_holochain_instance("test_bad_entry", "alice");
    let result = make_test_call(&mut hc, "get_entry", r#"{"address":"aba"}"#);

    let expected_result: ZomeApiResult<Option<Entry>> =
        serde_json::from_str::<ZomeApiResult<Option<Entry>>>(&result.clone().unwrap().to_string())
            .unwrap();
    assert_eq!(expected_result.unwrap(), None)
}

#[test]
fn can_round_trip() {
    let (mut hc, test_logger, _) = start_holochain_instance("can_round_trip", "alice");
    let result = make_test_call(
        &mut hc,
        "send_tweet",
        r#"{ "author": "bob", "content": "had a boring day" }"#,
    );
    assert_eq!(
        result.unwrap(),
        JsonString::from_json("{\"first\":\"bob\",\"second\":\"had a boring day\"}"),
    );

    let test_logger = test_logger.lock().unwrap();

    println!("{:?}", *test_logger);
}

#[test]
fn can_get_entry_ok() {
    let (mut hc, _, _) = start_holochain_instance("can_get_entry_ok", "alice");
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
fn can_get_entry_bad() {
    let (mut hc, _, _) = start_holochain_instance("can_get_entry_bad", "alice");
    // Call the exposed wasm function that calls the Commit API function

    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &example_valid_entry_params(),
    );
    let expected: ZomeApiResult<Address> = Ok(example_valid_entry_address());
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(result.unwrap(), JsonString::from(expected));
    // test the case with a bad address
    let result = make_test_call(
        &mut hc,
        "check_get_entry_result",
        &String::from(JsonString::from(json!(
            {"entry_address": Address::from("QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx")}
        ))),
    );
    assert!(result.is_ok(), "result = {:?}", result);
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
    assert!(result.is_ok(), "result = {:?}", result);
    let expected: ZomeApiResult<Option<Entry>> = Ok(None);
    assert_eq!(result.unwrap(), JsonString::from(expected));
}

#[test]
fn can_commit_entry() {
    let (mut hc, _, _) = start_holochain_instance("can_commit_entry", "alice");

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
fn can_return_empty_string_as_validation_fail() {
    let (mut hc, _, _) =
        start_holochain_instance("can_return_empty_string_as_validation_fail", "alice");

    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry",
        &String::from(JsonString::from(empty_string_validation_fail_entry())),
    );
    let path = PathBuf::new()
        .join("crates")
        .join("core")
        .join("src")
        .join("nucleus")
        .join("ribosome")
        .join("runtime.rs");
    let path_string = path
        .as_path()
        .to_str()
        .expect("path should have been created");
    let formatted_path_string = path_string.replace("\\", &vec!["\\", "\\", "\\", "\\"].join(""));
    let expected_substr = format!("{{\"Internal\":\"{{\\\"kind\\\":{{\\\"ValidationFailed\\\":\\\"\\\"}},\\\"file\\\":\\\"{}\\\"",formatted_path_string);
    let result_str = result.unwrap().to_string();

    assert!(result_str.contains(&expected_substr));
}
