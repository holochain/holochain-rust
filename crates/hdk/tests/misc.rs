extern crate holochain_conductor_lib;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate tempfile;
extern crate test_utils;
extern crate url;
#[macro_use]
extern crate serde_json;
extern crate hdk;
extern crate holochain_wasm_utils;

use hdk::error::ZomeApiResult;

use holochain_core::signal::{Signal, UserSignal};
use holochain_core_types::{
    entry::{entry_type::test_app_entry_type, Entry},
    error::{RibosomeEncodedValue, RibosomeEncodingBits},
};

use holochain_json_api::json::JsonString;
use holochain_persistence_api::{
    cas::content::{Address, AddressableContent},
    hash::HashString,
};

use std::path::PathBuf;
use test_utils::{
    example_valid_entry_address, example_valid_entry_params, make_test_call,
    start_holochain_instance, TestEntry,
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
fn can_use_globals() {
    let (mut hc, _, _) = start_holochain_instance("can_use_globals", "alice");
    // Call the exposed wasm function that calls the debug API function for printing all GLOBALS
    let result = make_test_call(&mut hc, "check_global", r#"{}"#);
    assert_eq!(
        result.clone(),
        Ok(JsonString::from(HashString::from(
            "HcScig9W6jFsqqfdohnIQrED7bJ99aezxvQRx5XMVjvge5p8x7sT4HcP4Bhceuz"
        ))),
        "result = {:?}",
        result
    );
}

#[test]
fn can_commit_entry_macro() {
    let (mut hc, _, _) = start_holochain_instance("can_commit_entry_macro", "alice");
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
fn can_invalidate_invalid_commit() {
    let (mut hc, _, _) = start_holochain_instance("can_invalidate_invalid_commit", "alice");
    // Call the exposed wasm function that calls the Commit API function
    let result = make_test_call(
        &mut hc,
        "check_commit_entry_macro",
        &json!({"entry":
            Entry::App(
                test_app_entry_type().into(),
                TestEntry {
                    stuff: "FAIL".into(),
                }.into(),
            )
        })
        .to_string(),
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
    let error_string = format!("{{\"Err\":{{\"Internal\":\"{{\\\"kind\\\":{{\\\"ValidationFailed\\\":\\\"FAIL content is not allowed\\\"}},\\\"file\\\":\\\"{}\\\",\\\"line\\\":\\\"",formatted_path_string);
    assert!(result.is_ok(), "result = {:?}", result);
    assert!(result.unwrap().to_string().contains(&error_string));
}

#[test]
fn has_populated_validation_data() {
    let (mut hc, _, _) = start_holochain_instance("has_populated_validation_data", "alice");

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
        JsonString::from_json("{\"Err\":{\"Internal\":\"{\\\"package\\\":{\\\"chain_header\\\":{\\\"entry_type\\\":{\\\"App\\\":\\\"validation_package_tester\\\"},\\\"entry_address\\\":\\\"QmYQPp1fExXdKfmcmYTbkw88HnCr3DzMSFUZ4ncEd9iGBY\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmSQqKHPpYZbafF7PXPKx31UwAbNAmPVuSHHxcBoDcYsci\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},\\\"source_chain_entries\\\":[{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"\\\\\\\"non fail\\\\\\\"\\\",\\\"entry_type\\\":\\\"testEntryType\\\"},{\\\"value\\\":\\\"alex\\\",\\\"entry_type\\\":\\\"%agent_id\\\"}],\\\"source_chain_headers\\\":[{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"link_same_type\\\":\\\"QmRHUwiUuFJiMyRmKaA1U49fXEnT8qbZMoj2V9maa4Q3JE\\\",\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":{\\\"App\\\":\\\"testEntryType\\\"},\\\"entry_address\\\":\\\"QmXxdzM9uHiSfV1xDwUxMm5jX4rVU8jhtWVaeCzjkFW249\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmRYerwRRXYxmYoxq1LTZMVVRfjNMAeqmdELTNDxURtHEZ\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"},{\\\"entry_type\\\":\\\"AgentId\\\",\\\"entry_address\\\":\\\"QmQw3V41bAWkQA9kwpNfU3ZDNzr9YW4p9RV4QHhFD3BkqA\\\",\\\"entry_signature\\\":\\\"\\\",\\\"link\\\":\\\"QmQJxUSfJe2QoxTyEwKQX9ypbkcNv3cw1vasGTx1CUpJFm\\\",\\\"link_same_type\\\":null,\\\"timestamp\\\":\\\"\\\"}],\\\"custom\\\":null},\\\"sources\\\":[\\\"<insert your agent key here>\\\"],\\\"lifecycle\\\":\\\"Chain\\\",\\\"action\\\":\\\"Commit\\\"}\"}}"),
        result.unwrap(),
    );
    */
}

#[test]
fn can_check_query() {
    let (mut hc, _, _) = start_holochain_instance("can_check_query", "alice");

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
    let (mut hc, _, _) = start_holochain_instance("can_check_app_entry_address", "alice");

    let result = make_test_call(&mut hc, "check_app_entry_address", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);

    let expected: ZomeApiResult<Address> = Ok(Address::from(
        "QmSbNw63sRS4VEmuqFBd7kJT6V9pkEpMRMY2LWvjNAqPcJ",
    ));
    assert_eq!(result.unwrap(), JsonString::from(expected),);
}

#[test]
fn can_check_sys_entry_address() {
    let (mut hc, _, _) = start_holochain_instance("can_check_sys_entry_address", "alice");

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
#[cfg(feature = "broken-tests")]
//lib3h in memory specific test
fn can_send_and_receive() {
    let (mut hc, _, _) = start_holochain_instance("can_send_and_receive", "alice");
    let _endpoint = hc
        .context()
        .expect("context")
        .network()
        .lock()
        .as_ref()
        .unwrap()
        .p2p_endpoint();
    let result = make_test_call(&mut hc, "check_global", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
    let agent_id = result.unwrap().to_string();

    let (mut hc2, _, _) = start_holochain_instance("can_send_and_receive", "bob");
    let params = format!(r#"{{"to_agent": {}, "message": "TEST"}}"#, agent_id);
    let result = make_test_call(&mut hc2, "send_message", &params);
    assert!(result.is_ok(), "result = {:?}", result);

    let entry_committed_by_receive = Entry::App(
        "testEntryType".into(),
        TestEntry {
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
    let (mut hc, _, _) = start_holochain_instance("sleep_smoke_test", "alice");
    let result = make_test_call(&mut hc, "sleep", r#"{}"#);
    assert!(result.is_ok(), "result = {:?}", result);
}

#[test]
fn show_env() {
    let (mut hc, _, _) = start_holochain_instance("show_env", "alice");
    let dna = hc.context().unwrap().get_dna().unwrap();
    let dna_address_string = dna.address().to_string();
    let dna_address = dna_address_string.as_str();
    let format   = format!(r#"{{"Ok":{{"dna_name":"TestApp","dna_address":"{}","agent_id":"{{\"nick\":\"show_env\",\"pub_sign_key\":\"HcSCIBgTFMzn8vz5ogz5eW87h9nf5eqpdsJOKJ47ZRDopz74HihmraGXio74e6i\"}}","agent_address":"HcSCIBgTFMzn8vz5ogz5eW87h9nf5eqpdsJOKJ47ZRDopz74HihmraGXio74e6i","cap_request":{{"cap_token":"QmSbEonVd9pmUxQqAS87a6CkdMPUsrjYLshW9eYz3wJkZ1","provenance":["HcSCIBgTFMzn8vz5ogz5eW87h9nf5eqpdsJOKJ47ZRDopz74HihmraGXio74e6i","FxhnQJzPu+TPqJHCtT2e5CNMky2YnnLXtABMJyNhx5SyztyeuKU/zxS4a1e8uKdPYT5N0ldCcLgpITeHfB7dAg=="]}},"properties":"{{}}"}}}}"#,dna_address);
    let json_result = Ok(JsonString::from_json(&format));

    let result = make_test_call(&mut hc, "show_env", r#"{}"#);

    assert_eq!(result, json_result)
}

#[test]
fn test_signal() {
    let (mut hc, _, signal_receiver) = start_holochain_instance("test_signal", "alice");
    let params = r#"{"message":"test message"}"#;
    let result = make_test_call(&mut hc, "emit_signal", &params);
    assert!(result.is_ok());
    assert!(signal_receiver
        .iter()
        .find(|recv| match recv {
            Signal::User(recieved_signal) => {
                recieved_signal
                    == &UserSignal {
                        name: String::from("test-signal"),
                        arguments: JsonString::from(r#"{"message":"test message"}"#)
                    }
            }
            _ => false,
        })
        .is_some());
}

#[test]
fn test_get_entry_properties() {
    let (mut hc, _, _) = start_holochain_instance("test_get_entry_properties", "alice");
    let result = make_test_call(
        &mut hc,
        "get_entry_properties",
        r#"{"entry_type_string": "testEntryType"}"#,
    );
    assert_eq!(
        result,
        Ok(JsonString::from(r#"{"Ok":"test-properties-string"}"#)),
        "result = {:?}",
        result,
    );
}
