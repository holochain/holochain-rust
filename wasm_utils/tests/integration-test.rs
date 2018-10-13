extern crate holochain_agent;
extern crate holochain_core;
extern crate test_utils;
extern crate holochain_core_api;
extern crate holochain_wasm_utils;
extern crate serde_json;

use holochain_core::{
    context::Context, logger::Logger,
    persister::SimplePersister,
};
use holochain_core_api::Holochain;
use std::sync::{Arc, Mutex};
use test_utils::{
    create_test_cap_with_fn_name,
    create_test_dna_with_cap,
    create_wasm_from_file,
};
use holochain_agent::Agent;
use holochain_wasm_utils::error::*;

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
    let agent = Agent::from_string(agent_name.to_string());
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


#[test]
fn can_call_serialize_test() {
    // Setup the holochain instance
    let wasm = create_wasm_from_file(
        "wasm-test/integration-test/target/wasm32-unknown-unknown/debug/integration_test.wasm",
    );
    let capability = create_test_cap_with_fn_name("test_serialize");
    let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

    let (context, test_logger) = test_context_and_logger("alex");
    let mut hc = Holochain::new(dna.clone(), context).unwrap();

    // Run the holochain instance
    hc.start().expect("couldn't start");
    // @TODO don't use history length in tests
    // @see https://github.com/holochain/holochain-rust/issues/195
    assert_eq!(hc.state().unwrap().history.len(), 3);

    // Call the exposed wasm function that calls the Commit API function
    let result = hc.call("test_zome", "test_cap", "test_serialize", r#"{}"#);
    let error_report: RibosomeErrorReport = serde_json::from_str(&result.clone().unwrap()).unwrap();
    println!("{}", error_report.to_string());
    assert_eq!("{\"value\":\"fish\"}", result.unwrap());

    let test_logger = test_logger.lock().unwrap();
    assert_eq!(
        format!("{:?}", *test_logger),
        "TestLogger { log: [\"TestApp instantiated\"] }",
    );
}
