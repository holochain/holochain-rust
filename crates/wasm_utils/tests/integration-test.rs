extern crate holochain_conductor_lib;
extern crate holochain_core;
extern crate holochain_core_types;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_json_derive;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate holochain_wasm_utils;
extern crate serde_json;
extern crate tempfile;
extern crate test_utils;

use holochain_conductor_lib::error::{HolochainInstanceError, HolochainResult};
use holochain_core_types::{
    bits_n_pieces::U16_MAX,
    error::{CoreError, HolochainError},
};

use holochain_json_api::{
    error::JsonError,
    json::{JsonString, RawString},
};

use holochain_wasm_utils::{memory::MemoryInt, wasm_target_dir};
use std::{convert::TryFrom, path::PathBuf};
use test_utils::hc_setup_and_call_zome_fn;

fn call_zome_function_with_hc<J: Into<JsonString>>(
    fn_name: &str,
    params: J,
) -> HolochainResult<JsonString> {
    let mut wasm_path = PathBuf::new();
    let wasm_path_component: PathBuf =
        [String::from("wasm-test"), String::from("integration-test")]
            .iter()
            .collect();
    wasm_path.push(wasm_target_dir(
        &String::from("wasm_utils").into(),
        &wasm_path_component,
    ));
    let wasm_file_path: PathBuf = [
        String::from("wasm32-unknown-unknown"),
        String::from("release"),
        String::from("wasm_integration_test.wasm"),
    ]
    .iter()
    .collect();
    wasm_path.push(wasm_file_path);
    hc_setup_and_call_zome_fn(&wasm_path, fn_name, params)
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
        Ok(JsonString::from_json("fish")),
        call_zome_function_with_hc("store_string", RawString::from("")),
    );
}

#[test]
fn store_string_err_test() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed("Zome function failure: Out of memory".into())
        )),
        call_zome_function_with_hc("store_string_err", RawString::from("")),
    );
}

#[test]
fn load_string_test() {
    assert_eq!(
        Ok(JsonString::from_json("fish")),
        call_zome_function_with_hc("load_string", RawString::from("")),
    );
}

#[test]
fn stacked_strings_test() {
    assert_eq!(
        Ok(JsonString::from_json("first")),
        call_zome_function_with_hc("stacked_strings", RawString::from("")),
    );
}

#[test]
fn big_string_input_static_test() {
    let s = "foobarbazbing".repeat(U16_MAX as usize);
    assert_eq!(
        JsonString::from(
            String::from(JsonString::from(RawString::from(s.clone()))).len() as MemoryInt
        ),
        call_zome_function_with_hc("big_string_input", RawString::from(s)).unwrap(),
    );
}

#[test]
/// test that we can send a big string as input to a zome function
/// at this point it is fine to preinitialize multiple wasm pages (not testing dynamic)
fn big_string_process_static_test() {
    // assert happens inside the zome because this test shows internal processing
    call_zome_function_with_hc("big_string_process_static", RawString::from("")).unwrap();
}

#[test]
/// test that we can send a big string as input to a zome function
/// at this point it is fine to preinitialize multiple wasm pages (not testing dynamic)
fn big_string_output_static_test() {
    let s = call_zome_function_with_hc("big_string_output_static", RawString::from("")).unwrap();
    let expected = "(ಥ⌣ಥ)".repeat(U16_MAX as usize);
    assert_eq!(String::from(s).len(), expected.len());
    assert_eq!(
        Ok(JsonString::from_json(&expected)),
        call_zome_function_with_hc("big_string_output_static", RawString::from("")),
    );
}

#[test]
pub fn round_trip_foo_test() {
    assert_eq!(
        Ok(JsonString::from_json("foo")),
        call_zome_function_with_hc("round_trip_foo", RawString::from("")),
    );
}

#[test]
fn error_report_test() {
    let call_result = call_zome_function_with_hc("error_report", RawString::from("")).unwrap();
    let core_err = CoreError::try_from(call_result).unwrap();
    assert!(core_err
        .to_string()
        .contains("Holochain Core error: Zome assertion failed: `false`"));
}

#[test]
fn store_as_json_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("fish"))),
        call_zome_function_with_hc("store_as_json", RawString::from("")),
    );
}

#[test]
fn store_load_struct_as_json_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("store_struct_as_json", RawString::from("")),
    );
}

#[test]
fn load_json_struct_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("load_json_struct", RawString::from("")),
    );
}

#[test]
fn stacked_json_struct_test() {
    assert_eq!(
        Ok(JsonString::from(fake_test_struct())),
        call_zome_function_with_hc("stacked_json_struct", RawString::from("")),
    );
}

#[test]
fn stacked_json_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("first"))),
        call_zome_function_with_hc("stacked_json", RawString::from(""))
    );
}

#[test]
fn call_store_as_json_err() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed("Zome function failure: Out of memory".into())
        )),
        call_zome_function_with_hc("store_json_err", RawString::from("")),
    );
}

#[test]
fn load_json_err_test() {
    assert_eq!(
        Err(HolochainInstanceError::from(
            HolochainError::RibosomeFailed("Zome function failure: Unspecified".into())
        )),
        call_zome_function_with_hc("load_json_err", RawString::from("")),
    );
}

#[test]
fn stacked_mix_test() {
    assert_eq!(
        Ok(JsonString::from(RawString::from("third"))),
        call_zome_function_with_hc("stacked_mix", RawString::from("")),
    );
}

// ===============================================================================================
// END MEMORY
// -----------------------------------------------------------------------------------------------
