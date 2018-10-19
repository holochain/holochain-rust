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

use holochain_agent::Agent;
use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
use holochain_core::{
    context::Context, logger::Logger, nucleus::ZomeFnResult, persister::SimplePersister,
};
use holochain_core_api::Holochain;
use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::error::*;
use std::sync::{Arc, Mutex};

use tempfile::tempdir;
use test_utils::{create_test_cap_with_fn_name, create_test_dna_with_cap, create_wasm_from_file};

#[derive(Clone, Debug)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }
}

/// create a test context and TestLogger pair so we can use the logger in assertions
pub fn create_test_context(agent_name: &str) -> Arc<Context> {
    let agent = Agent::from(agent_name.to_string());
    let logger = Arc::new(Mutex::new(TestLogger { log: Vec::new() }));

    Arc::new(
        Context::new(
            agent,
            logger.clone(),
            Arc::new(Mutex::new(SimplePersister::new())),
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
            EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
        ).unwrap(),
    )
}

// Function called at start of all unit tests:
//   Startup holochain and do a call on the specified wasm function.
pub fn call_zome_function_with_hc(fn_name: &str) -> ZomeFnResult {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/release/wasm_integration_test.wasm",
    );
    let capability = create_test_cap_with_fn_name(fn_name);
    let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

    let context = create_test_context("alex");
    let mut hc = Holochain::new(dna.clone(), context).unwrap();

    // Run the holochain instance
    hc.start().expect("couldn't start");
    // Call the exposed wasm function
    return hc.call("test_zome", "test_cap", fn_name, r#"{}"#);
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
        HolochainError::RibosomeFailed(RibosomeErrorCode::OutOfMemory.to_string()),
        call_result.err().unwrap(),
    );
}

#[test]
fn call_store_as_json_err() {
    let call_result = call_zome_function_with_hc("test_store_as_json_err");
    assert_eq!(
        HolochainError::RibosomeFailed(RibosomeErrorCode::OutOfMemory.to_string()),
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
