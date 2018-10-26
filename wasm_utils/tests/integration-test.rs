#![feature(try_from)]
extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_wasm_utils;
extern crate serde_json;
extern crate holochain_cas_implementations;
extern crate tempfile;
extern crate test_utils;

<<<<<<< HEAD
use holochain_agent::Agent;
use holochain_core::{context::Context, logger::Logger, persister::SimplePersister};
use holochain_core_api::Holochain;
use holochain_core_types::{
    error::{HolochainError, *},
    json::{JsonString, RawString},
};
use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
};
use test_utils::{create_test_cap_with_fn_name, create_test_dna_with_cap, create_wasm_from_file};

=======
use holochain_core::logger::Logger;
use holochain_core_api::error::{HolochainInstanceError, HolochainResult};
use holochain_core_types::error::{HolochainError, RibosomeErrorCode, RibosomeErrorReport};
use test_utils::hc_setup_and_call_zome_fn;
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
#[derive(Clone, Debug)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }

    fn dump(&self) -> String {
        format!("{:?}", self.log)
    }
}

fn call_zome_function_with_hc(fn_name: &str) -> HolochainResult<String> {
    hc_setup_and_call_zome_fn(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/release/wasm_integration_test.wasm",
        fn_name)
}

<<<<<<< HEAD
pub fn launch_hc_with_integration_test_wasm(
    fn_name: &str,
    fn_arg: &str,
) -> (Result<JsonString, HolochainError>, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/release/integration_test.wasm",
    );
    let capability = create_test_cap_with_fn_name(fn_name);
    let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
=======
#[test]
fn can_return_error_report() {
    let call_result = call_zome_function_with_hc("test_error_report");
    let error_report: RibosomeErrorReport =
        serde_json::from_str(&call_result.clone().unwrap()).unwrap();
    assert_eq!("Zome assertion failed: `false`", error_report.description);
}
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6

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
<<<<<<< HEAD
fn can_return_error_report() {
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_error_report", r#"{}"#);
    // Verify result
    let error_report =
        RibosomeErrorReport::from(JsonString::from(RawString::from(result.clone().unwrap())));
    assert_eq!("Zome assertion failed: `false`", error_report.description);
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}

/// TODO #486 - load and store string from wasm memory
//#[test]
//fn call_store_string_ok() {
//    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_store_string_ok", r#"{}"#);
//    println!("result = {:?}", result);
//    // Verify result
//    assert_eq!("some string", result.unwrap());
//    // Verify logs
//    let test_logger = test_logger.lock().unwrap();
//    assert_eq!(
//        format!("{:?}", *test_logger),
//        "TestLogger { log: [\"TestApp instantiated\"] }",
//    );
//}

#[test]
fn call_store_json_ok() {
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_store_json_ok", r#"{}"#);
    // Verify result
    assert_eq!(JsonString::from("{\"value\":\"fish\"}"), result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
=======
fn call_store_as_json_obj_ok() {
    let call_result = call_zome_function_with_hc("test_store_as_json_obj_ok");
    assert_eq!("{\"value\":\"fish\"}", call_result.unwrap());
}

#[test]
fn call_store_string_err() {
    let call_result = call_zome_function_with_hc("test_store_string_err");
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(
            RibosomeErrorCode::OutOfMemory.to_string()
        )),
        call_result.err().unwrap(),
    );
}

#[test]
<<<<<<< HEAD
fn call_store_json_err() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_store_json_err", r#"{}"#);
    // Verify result
    assert!(result.is_ok());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\", \"Zome Function did not allocate memory: \\\'test_store_json_err\\\' return code: Out of memory\"] }",
=======
fn call_store_as_json_err() {
    let call_result = call_zome_function_with_hc("test_store_as_json_err");
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(
            RibosomeErrorCode::OutOfMemory.to_string()
        )),
        call_result.err().unwrap(),
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    );
}

#[test]
fn call_load_json_from_raw_ok() {
<<<<<<< HEAD
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_load_json_from_raw_ok", r#"{}"#);
    // Verify result
    assert_eq!(JsonString::from("Success"), result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\", \"Zome Function did not allocate memory: \\\'test_load_json_from_raw_ok\\\' return code: Success\"] }",
    );
=======
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_ok");
    assert_eq!("", call_result.unwrap());
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
}

#[test]
fn call_load_json_from_raw_err() {
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_err");
    assert_eq!(
<<<<<<< HEAD
        JsonString::try_from(RibosomeErrorCode::ArgumentDeserializationFailed).unwrap(),
        result.unwrap()
    );
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
=======
        json!(RibosomeErrorCode::ArgumentDeserializationFailed.to_string()).to_string(),
        call_result.unwrap()
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
    );
}

#[test]
fn call_load_json_ok() {
<<<<<<< HEAD
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_load_json_ok", r#"{}"#);
    // Verify result
    assert_eq!(JsonString::from("{\"value\":\"fish\"}"), result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
=======
    let call_result = call_zome_function_with_hc("test_load_json_ok");
    assert_eq!("{\"value\":\"fish\"}", call_result.unwrap());
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
}

#[test]
fn call_load_json_err() {
<<<<<<< HEAD
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_load_json_err", r#"{}"#);
    // Verify result
    assert_eq!(JsonString::from("\"Unspecified\""), result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
=======
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
>>>>>>> da8059ec89cfc40bb22f543dba06c32e7fd60ba6
}
