#![feature(try_from)]
extern crate holochain_core_types;
#[macro_use]
extern crate holochain_core_types_derive;
#[macro_use]
extern crate serde_derive;
extern crate holochain_wasm_utils;
extern crate serde_json;

use holochain_core_types::{
    cas::content::Address, error::HolochainError,
    error::RibosomeReturnCode, error::ZomeApiInternalResult, json::JsonString, json::RawString,
};
use std::convert::TryInto;
use holochain_core_types::entry::Entry;

//-------------------------------------------------------------------------------------------------
// HC DEBUG Function Call
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_debug(encoded_allocation_of_input: i32) -> i32;
}

/// Call HC API DEBUG function with proper input struct: a string
/// return error code
fn hdk_debug(mem_stack: &mut WasmStack, json_string: &JsonString) {
    // Write input string on stack
    let maybe_allocation = store_as_json(mem_stack, json_string.to_owned());
    if let Err(_) = maybe_allocation {
        return;
    }
    let allocation_of_input = maybe_allocation.unwrap();
    // Call WASMI-able DEBUG
    unsafe {
        hc_debug(allocation_of_input.encode() as i32);
    }
    // Free input allocation and all allocations made inside print()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
}

//-------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_hello(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    hdk_debug(
        &mut mem_stack,
        &JsonString::from(RawString::from("Hello world!")),
    );
    i32::from(RibosomeReturnCode::Success)
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_multiple(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("Hello")));
    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("world")));
    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("!")));
    i32::from(RibosomeReturnCode::Success)
}

//-------------------------------------------------------------------------------------------------
//  More tests
//-------------------------------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn debug_stacked_hello(encoded_allocation_of_input: usize) -> i32 {
    #[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
    struct TestStruct {
        value: String,
    }

    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    let fish = store_as_json_into_encoded_allocation(
        &mut mem_stack,
        TestStruct {
            value: "fish".to_string(),
        },
    );
    hdk_debug(&mut mem_stack, &JsonString::from("disruptive debug log"));
    fish
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Successful
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_commit_entry(encoded_allocation_of_input: i32) -> i32;
}

/// Call HC API COMMIT function with proper input struct
/// return address of entry added source chain
fn hdk_commit(
    mem_stack: &mut WasmStack,
    entry_type_name: &str,
    entry_value: &str,
) -> Result<Address, String> {
    // Put args in struct and serialize into memory
    let entry = Entry::App(
        entry_type_name.to_owned().into(),
        entry_value.to_owned().into(),
    );
    let allocation_of_input = store_as_json(mem_stack, JsonString::from(entry))?;

    // Call WASMI-able commit
    let encoded_allocation_of_result: i32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
    }
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    match JsonString::from(result.value).try_into() {
        Ok(address) => Ok(address),
        Err(hc_err) => Err(hc_err.into()),
    }
}

//-------------------------------------------------------------------------------------------------
// HC COMMIT Function Call - Fail
//-------------------------------------------------------------------------------------------------

// Simulate error in commit function by inputing output struct as input
fn hdk_commit_fail(mem_stack: &mut WasmStack) -> Result<Address, String> {
    // Put args in struct and serialize into memory
    let input = ZomeApiInternalResult::failure(Address::from("whatever"));
    let allocation_of_input = store_as_json(mem_stack, input)?;

    // Call WASMI-able commit
    let encoded_allocation_of_result: i32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
    }
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;

    // Free result & input allocations and all allocations made inside commit()
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    let address = JsonString::from(result.value).try_into()?;

    Ok(address)
}

//--------------------------------------------------------------------------------------------------
// Test roundtrip function
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct InputStruct {
    input_int_val: u8,
    input_str_val: String,
}

#[derive(Serialize, Default, Deserialize, Debug, DefaultJson)]
struct OutputStruct {
    input_int_val_plus2: u8,
    input_str_val_plus_dog: String,
}

/// Create output out of some modification of input
fn test_inner(input: InputStruct) -> OutputStruct {
    OutputStruct {
        input_int_val_plus2: input.input_int_val + 2,
        input_str_val_plus_dog: format!("{}.puppy", input.input_str_val),
    }
}

//-------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn commit_test(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    let result = hdk_commit(&mut mem_stack, "testEntryType", "hello");
    store_as_json_into_encoded_allocation(&mut mem_stack, result)
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn commit_fail_test(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    let result = hdk_commit_fail(&mut mem_stack);
    store_as_json_into_encoded_allocation(&mut mem_stack, result)
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(_encoded_allocation_of_input: u32) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    encoded_allocation_of_input: usize,
) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    store_string_into_encoded_allocation(&mut mem_stack, "\"ChainFull\"")
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn round_trip_test(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack =
        WasmStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
    let input = load_json(encoded_allocation_of_input as u32).unwrap();
    let output = test_inner(input);
    return store_as_json_into_encoded_allocation(&mut mem_stack, JsonString::from(output));
}
