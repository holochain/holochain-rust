#![feature(try_from)]
extern crate holochain_cas_implementations;
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;
extern crate holochain_wasm_utils;
extern crate serde_json;
extern crate tempfile;
extern crate test_utils;

use holochain_container_api::error::{HolochainInstanceError, HolochainResult};
use holochain_core::logger::Logger;
use holochain_core_types::{
    error::{CoreError, HolochainError, RibosomeErrorCode},
    json::{default_try_from_json, JsonString, RawString},
};
use std::{convert::TryFrom, error::Error};
use test_utils::hc_setup_and_call_zome_fn;
use test_utils::wasm_target_dir;

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

fn call_zome_function_with_hc(fn_name: &str) -> HolochainResult<JsonString> {
    hc_setup_and_call_zome_fn(
        &format!(
            "{}/wasm32-unknown-unknown/release/wasm_integration_test.wasm",
            wasm_target_dir("wasm-test/integration-test/target"),
        ),
        fn_name,
    )
}

#[test]
fn can_return_core_error_test() {
    let call_result = call_zome_function_with_hc("test_error_report").unwrap();
    let core_err = CoreError::try_from(call_result).unwrap();
    assert_eq!("Zome assertion failed: `false`", core_err.description());
}

#[test]
fn call_store_string_ok() {
    let call_result = call_zome_function_with_hc("test_store_string_ok");
    assert_eq!(JsonString::from("fish"), call_result.unwrap());
}

#[test]
fn call_store_as_json_str_ok() {
    let call_result = call_zome_function_with_hc("test_store_as_json_str_ok");
    assert_eq!(
        JsonString::from(RawString::from("fish")),
        call_result.unwrap()
    );
}

#[test]
fn call_store_as_json_obj_ok() {
    let call_result = call_zome_function_with_hc("test_store_as_json_obj_ok");
    assert_eq!(
        JsonString::from("{\"value\":\"fish\",\"list\":[\"hello\",\"world!\"]}"),
        call_result.unwrap()
    );
}

#[test]
fn call_store_string_err() {
    let call_result = call_zome_function_with_hc("test_store_string_err");
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(
            RibosomeErrorCode::OutOfMemory.to_string()
        )),
        call_result.err().unwrap(),
    );
}

#[test]
fn call_store_as_json_err() {
    let call_result = call_zome_function_with_hc("test_store_as_json_err");
    assert_eq!(
        HolochainInstanceError::from(HolochainError::RibosomeFailed(
            RibosomeErrorCode::OutOfMemory.to_string()
        )),
        call_result.err().unwrap(),
    );
}

#[test]
fn call_load_json_from_raw_ok() {
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_ok");
    assert_eq!(JsonString::null(), call_result.unwrap());
}

#[test]
fn call_load_json_from_raw_err() {
    let call_result = call_zome_function_with_hc("test_load_json_from_raw_err");
    assert_eq!(
        JsonString::from(RibosomeErrorCode::ArgumentDeserializationFailed.to_string()),
        call_result.unwrap()
    );
}

#[test]
fn call_load_json_ok() {
    let call_result = call_zome_function_with_hc("test_load_json_ok");
    assert_eq!(
        JsonString::from("{\"value\":\"fish\",\"list\":[\"hello\",\"world!\"]}"),
        call_result.unwrap()
    );
}

#[test]
fn call_load_json_err_test() {
    #[derive(Serialize, Deserialize, Debug, DefaultJson)]
    struct TestStruct {
        value: String,
        list: Vec<String>,
    }
    type TestResult = Result<TestStruct, HolochainError>;

    let try_result = call_zome_function_with_hc("test_load_json_err");

    match try_result {
        Ok(result) => {
            let test_result: TestResult = default_try_from_json(result).unwrap();
            match test_result {
                Err(e) => assert_eq!(HolochainError::Ribosome(RibosomeErrorCode::Unspecified), e),
                Ok(_) => unreachable!(),
            }
        }
        Err(_) => unreachable!(),
    }
}

#[test]
fn call_load_string_ok() {
    let call_result = call_zome_function_with_hc("test_load_string_ok");
    assert_eq!(JsonString::from("fish"), call_result.unwrap());
}

#[test]
fn call_load_string_err() {
    let call_result = call_zome_function_with_hc("test_load_string_err");
    assert_eq!(JsonString::from("Unspecified"), call_result.unwrap());
}

#[test]
fn call_stacked_strings() {
    let call_result = call_zome_function_with_hc("test_stacked_strings");
    assert_eq!(JsonString::from("first"), call_result.unwrap());
}

#[test]
fn call_stacked_json_str() {
    let call_result = call_zome_function_with_hc("test_stacked_json_str");
    assert_eq!(JsonString::from("first"), call_result.unwrap());
}

#[test]
fn call_stacked_json_obj() {
    let call_result = call_zome_function_with_hc("test_stacked_json_obj");
    assert_eq!(
        JsonString::from("{\"value\":\"first\",\"list\":[\"hello\",\"world!\"]}"),
        call_result.unwrap()
    );
}

#[test]
fn call_stacked_mix() {
    let call_result = call_zome_function_with_hc("test_stacked_mix");
    assert_eq!(JsonString::from("third"), call_result.unwrap());
}
