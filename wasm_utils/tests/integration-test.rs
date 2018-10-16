extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_json;
extern crate test_utils;

use holochain_agent::Agent;
use holochain_core::{context::Context, logger::Logger, persister::SimplePersister};
use holochain_core_api::Holochain;
use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::error::*;
use std::sync::{Arc, Mutex};
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

/// create a test logger
pub fn test_logger() -> Arc<Mutex<TestLogger>> {
    Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
}

/// create a test context and TestLogger pair so we can use the logger in assertions
pub fn test_context_and_logger(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
    let agent = Agent::from(agent_name.to_string());
    let logger = test_logger();
    (
        Arc::new(Context::new(
            agent,
            logger.clone(),
            Arc::new(Mutex::new(SimplePersister::new())),
        )),
        logger,
    )
}

pub fn launch_hc_with_integration_test_wasm(
    fn_name: &str,
    fn_arg: &str,
) -> (Result<String, HolochainError>, Arc<Mutex<TestLogger>>) {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/release/integration_test.wasm",
    );
    let capability = create_test_cap_with_fn_name(fn_name);
    let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

    let (context, test_logger) = test_context_and_logger("alex");
    let mut hc = Holochain::new(dna.clone(), context).unwrap();

    // Run the holochain instance
    hc.start().expect("couldn't start");
    // Call the exposed wasm function
    let result = hc.call("test_zome", "test_cap", fn_name, fn_arg);
    return (result, test_logger);
}

#[test]
fn can_return_error_report() {
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_error_report", r#"{}"#);
    // Verify result
    let error_report: RibosomeErrorReport = serde_json::from_str(&result.clone().unwrap()).unwrap();
    assert_eq!("Zome assertion failed: `false`", error_report.description);
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}


#[test]
fn call_store_string_ok() {
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_store_string_ok", r#"{}"#);
    println!("result = {:?}", result);
    // Verify result
    assert_eq!("some string", result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}

#[test]
fn call_store_as_json_ok() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_store_as_json_ok", r#"{}"#);
    // Verify result
    assert_eq!("{\"value\":\"fish\"}", result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}

#[test]
fn call_store_as_json_err() {
    let (result, test_logger) = launch_hc_with_integration_test_wasm("test_store_as_json_err", r#"{}"#);
    // Verify result
    assert!(result.is_ok());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\", \"Zome Function \\\'test_store_as_json_err\\\' returned: Out of memory\"] }",
    );
}

#[test]
fn call_load_json_from_raw_ok() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_load_json_from_raw_ok", r#"{}"#);
    // Verify result
    assert_eq!("", result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\", \"Zome Function \\\'test_load_json_from_raw_ok\\\' returned: Success\"] }",
    );
}

#[test]
fn call_load_json_from_raw_err() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_load_json_from_raw_err", r#"{}"#);
    // Verify result
    assert_eq!(
        json!(RibosomeErrorCode::ArgumentDeserializationFailed.to_string()).to_string(),
        result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}

#[test]
fn call_load_json_ok() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_load_json_ok", r#"{}"#);
    // Verify result
    assert_eq!("{\"value\":\"fish\"}", result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}

#[test]
fn call_load_json_err() {
    let (result, test_logger) =
        launch_hc_with_integration_test_wasm("test_load_json_err", r#"{}"#);
    // Verify result
    assert_eq!("\"Unspecified\"", result.unwrap());
    // Verify logs
    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}
