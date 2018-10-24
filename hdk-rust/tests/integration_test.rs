extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_dna;
extern crate test_utils;
extern crate backtrace;

use holochain_core_types::entry_type::test_entry_type;
use holochain_core_types::cas::content::AddressableContent;
use holochain_core_types::entry::test_entry_a;
use holochain_core_types::entry::Entry;
use holochain_core_types::entry::SerializedEntry;
use holochain_core_api::*;
use holochain_core_types::json::{JsonString, RawString};
use holochain_core_types::hash::HashString;
use holochain_dna::zome::capabilities::{Capability, FnDeclaration};
use std::sync::{Arc, Mutex};
use test_utils::*;
use backtrace::Backtrace;

pub fn create_test_cap_with_fn_names(fn_names: Vec<&str>) -> Capability {
    let mut capability = Capability::new();

    for fn_name in fn_names {
        let mut fn_decl = FnDeclaration::new();
        fn_decl.name = String::from(fn_name);
        capability.functions.push(fn_decl);
    }
    capability
}

fn start_holochain_instance() -> (Holochain, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm =
        create_wasm_from_file("wasm-test/target/wasm32-unknown-unknown/release/test_globals.wasm");
    let capabability = create_test_cap_with_fn_names(vec![
        "check_global",
        "check_commit_entry",
        "check_commit_entry_macro",
        "check_get_entry",
        "send_tweet",
    ]);
    let dna = create_test_dna_with_cap("test_zome", "test_cap", &capabability, &wasm);

    let (context, test_logger) = test_context_and_logger("alex");
    let mut hc = Holochain::new(dna.clone(), context).unwrap();

    // Run the holochain instance
    hc.start().expect("couldn't start");
    (hc, test_logger)
}

#[test]
fn can_use_globals() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the debug API function for printing all GLOBALS
    let result = hc.call("test_zome", "test_cap", "check_global", r#"{}"#);
    assert_eq!(
        result.clone(),
        Ok(JsonString::from(HashString::from(
            "FIXME-app_agent_latest_hash"
        ))),
        "result = {:?}",
        result
    );
}

#[test]
fn can_commit_entry() {
    let (mut hc, _) = start_holochain_instance();

    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry",
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            format!("{{\"address\":\"{}\"}}", String::from(SerializedEntry::from(test_entry_a()).address()))
        ),
    );
}

#[test]
fn can_commit_entry_macro() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        // this works because the macro names the args the same as the SerializedEntry fields
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            format!("{{\"ok\":\"{}\"}}", String::from(SerializedEntry::from(test_entry_a()).address()))
        ),
    );
}

#[test]
fn can_round_trip() {
    let (mut hc, test_logger) = start_holochain_instance();
    let result = hc.call(
        "test_zome",
        "test_cap",
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
fn can_get_entry() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        &String::from(JsonString::from(SerializedEntry::from(test_entry_a()))),
    );
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from(
            format!("{{\"ok\":\"{}\"}}", String::from(SerializedEntry::from(test_entry_a()).address()))
        ),
    );

    // let result = hc.call(
    //     "test_zome",
    //     "test_cap",
    //     "check_get_entry",
    //     &format!("{{\"entry_address\":\"{}\"}}", test_entry_a().address()),
    // );
    // println!("\t can_get_entry result = {:?}", result);
    // assert!(result.is_ok(), "\t result = {:?}", result);
    // assert_eq!(
    //     result.unwrap(),
    //     JsonString::from("{\"ok\":{\"entry\":{\"value\":\"\\\"test entry value\\\"\",\"entry_type\":\"testEntryType\"}}}")
    // );
    //
    // // test the case with a bad hash
    // let result = hc.call(
    //     "test_zome",
    //     "test_cap",
    //     "check_get_entry",
    //     r#"{"entry_address":"QmbC71ggSaEa1oVPTeNN7ZoB93DYhxowhKSF6Yia2Vjxxx"}"#,
    // );
    // println!("\t can_get_entry result = {:?}", result);
    // assert!(result.is_ok(), "\t result = {:?}", result);
    // assert_eq!(
    //     result.unwrap(),
    //     JsonString::from("{\"ok\":{\"entry\":null}}")
    // );
}

#[test]
fn can_invalidate_invalid_commit() {
    let (mut hc, _) = start_holochain_instance();
    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call(
        "test_zome",
        "test_cap",
        "check_commit_entry_macro",
        &String::from(JsonString::from(SerializedEntry::from(Entry::new(&test_entry_type(), &JsonString::from("{\"stuff\": \"FAIL\"}"))))),
        // r#"{ "entry_type_name": "testEntryType", "entry_content": "{\"stuff\": \"FAIL\"}" }"#,
    );
    println!("\t result = {:?}", result);
    assert!(result.is_ok(), "\t result = {:?}", result);
    assert_eq!(
        result.unwrap(),
        JsonString::from("{\"error\":{\"error\":Validation failed: FAIL content is not allowed}}"),
    );
}
