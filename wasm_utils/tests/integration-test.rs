extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_json;
extern crate holochain_cas_implementations;
extern crate tempfile;
extern crate test_utils;

use holochain_core_api::error::{HolochainResult, HolochainInstanceError};
use holochain_core::logger::Logger;
use holochain_core_types::error::{HolochainError, RibosomeErrorCode, RibosomeErrorReport};
use test_utils::hc_setup_and_call_zome_fn;
#[derive(Clone, Debug)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }
}

fn call_zome_function_with_hc(fn_name: &str) -> HolochainResult<String> {
    hc_setup_and_call_zome_fn(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/release/wasm_integration_test.wasm",
        fn_name)
}

#[test]
fn can_return_error_report() {
    let call_result = call_zome_function_with_hc("test_error_report");
    let error_report: RibosomeErrorReport =
        serde_json::from_str(&call_result.clone().unwrap()).unwrap();
    assert_eq!("Zome assertion failed: `false`", error_report.description);
}

#[test]
fn call_store_string_ok() {
    let call_result = call_zome_function_with_hc("test_store_string_ok");
    assert_eq!("fish", call_result.unwrap());
}

#[test]
fn call_store_as_json_str_ok() {
    let call_result = call_zome_function_with_hc("test_store_as_json_str_ok");
    assert_eq!("\"fish\"", call_result.unwrap());
}

#[test]
fn call_store_as_json_obj_ok() {
    let call_result = call_zome_function_with_hc("test_store_as_json_obj_ok");
    assert_eq!("{\"value\":\"fish\"}", call_result.unwrap());
}

#[test]
fn call_store_string_err() {
    let call_result = call_zome_function_with_hc("test_store_string_err");
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(RibosomeErrorCode::OutOfMemory.to_string())),
        call_result.err().unwrap(),
    );
}

#[test]
fn call_store_as_json_err() {
    let call_result = call_zome_function_with_hc("test_store_as_json_err");
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(RibosomeErrorCode::OutOfMemory.to_string())),
        call_result.err().unwrap(),
    );
}

#[test]
fn call_load_json_from_raw_ok() {
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_ok");
    assert_eq!("", call_result.unwrap());
}

#[test]
fn call_load_json_from_raw_err() {
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_err");
    assert_eq!(
        json!(RibosomeErrorCode::ArgumentDeserializationFailed.to_string()).to_string(),
        call_result.unwrap()
    );
}

#[test]
fn call_load_json_ok() {
    let call_result = call_zome_function_with_hc("test_load_json_ok");
    assert_eq!("{\"value\":\"fish\"}", call_result.unwrap());
}

#[test]
fn call_load_json_err() {
    let call_result = call_zome_function_with_hc("test_load_json_err");
    assert_eq!("\"Unspecified\"", call_result.unwrap());
}

#[test]
fn call_load_string_ok() {
    let call_result = call_zome_function_with_hc("test_load_string_ok");
    assert_eq!("fish", call_result.unwrap());
}

#[test]
fn call_load_string_err() {
    let call_result = call_zome_function_with_hc("test_load_string_err");
    assert_eq!("Unspecified", call_result.unwrap());
}

#[test]
fn call_stacked_strings() {
    let call_result = call_zome_function_with_hc("test_stacked_strings");
    assert_eq!("first", call_result.unwrap());
}

#[test]
fn call_stacked_json_str() {
    let call_result = call_zome_function_with_hc("test_stacked_json_str");
    assert_eq!("\"first\"", call_result.unwrap());
}

#[test]
fn call_stacked_json_obj() {
    let call_result = call_zome_function_with_hc("test_stacked_json_obj");
    assert_eq!("{\"value\":\"first\"}", call_result.unwrap());
}

#[test]
fn call_stacked_mix() {
    let call_result = call_zome_function_with_hc("test_stacked_mix");
    assert_eq!("third", call_result.unwrap());
}
