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
use holochain_core_types::{
    error::{CoreError, HolochainError, RibosomeEncodedValue, RibosomeErrorCode},
    json::{JsonString, RawString},
    bits_n_pieces::U16_MAX,
};
use holochain_wasm_utils::wasm_target_dir;
use std::convert::TryFrom;
use test_utils::hc_setup_and_call_zome_fn;

fn call_zome_function_with_hc(fn_name: &str) -> HolochainResult<JsonString> {
    hc_setup_and_call_zome_fn(
        &format!(
            "{}/wasm32-unknown-unknown/release/wasm_integration_test.wasm",
            wasm_target_dir("wasm_utils/", "wasm-test/integration-test/"),
        ),
        fn_name,
    )
}

#[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
struct TestStruct {
    value: String,
    list: Vec<String>,
}

fn fake_test_struct() -> TestStruct {
    TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    }
}

// ===============================================================================================
// START MEMORY
// -----------------------------------------------------------------------------------------------

// ===============================================================================================
// STRINGS
// -----------------------------------------------------------------------------------------------

#[test]
fn store_string_test() {
    assert_eq!(
        Ok(JsonString::from("fish")),
        call_zome_function_with_hc("store_string"),
    );
}

#[test]
fn store_string_err_test() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed(
                RibosomeEncodedValue::Failure(RibosomeErrorCode::OutOfMemory).into()
            )
        )),
        call_zome_function_with_hc("store_string_err"),
    );
}

#[test]
fn load_string_test() {
    assert_eq!(
        Ok(JsonString::from("fish")),
        call_zome_function_with_hc("load_string"),
    );
}

#[test]
fn stacked_strings_test() {
    assert_eq!(
        Ok(JsonString::from("first")),
        call_zome_function_with_hc("stacked_strings"),
    );
}

#[test]
fn big_string_test() {
    let s = call_zome_function_with_hc("big_string").unwrap();
    assert_eq!(
        String::from(s).len(),
        (U16_MAX * 20) as usize,
    );
    // assert_eq!(
    //     Ok(JsonString::from("z")),
    //     // Ok(JsonString::from("(┛ಠ_ಠ)┛彡┻━┻".repeat(U16_MAX as usize))),
    //     call_zome_function_with_hc("big_string"),
    // );
}

#[test]
pub fn round_trip_foo_test() {
    assert_eq!(
        Ok(JsonString::from("foo")),
        call_zome_function_with_hc("round_trip_foo"),
    );
}

#[test]
fn error_report_test() {
    let call_result = call_zome_function_with_hc("error_report").unwrap();
    let core_err = CoreError::try_from(call_result).unwrap();
    assert!(core_err
        .to_string()
        .contains("Holochain Core error: Zome assertion failed: `false`"));
}

#[test]
fn store_as_json_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("fish"))),
        call_zome_function_with_hc("store_as_json"),
    );
}

#[test]
fn store_load_struct_as_json_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("store_struct_as_json"),
    );
}

#[test]
fn load_json_struct_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("load_json_struct"),
    );
}

#[test]
fn stacked_json_struct_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("stacked_json_struct"),
    );
}

#[test]
fn stacked_json_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("first"))),
        call_zome_function_with_hc("stacked_json")
    );
}

#[test]
fn call_store_as_json_err() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed(RibosomeErrorCode::OutOfMemory.into())
        )),
        call_zome_function_with_hc("store_json_err"),
    );
}

#[test]
fn load_json_err_test() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed(RibosomeErrorCode::Unspecified.into())
        )),
        call_zome_function_with_hc("load_json_err"),
    );
}

#[test]
fn stacked_mix_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("third"))),
        call_zome_function_with_hc("stacked_mix"),
    );
}

// ===============================================================================================
// END MEMORY
// -----------------------------------------------------------------------------------------------
