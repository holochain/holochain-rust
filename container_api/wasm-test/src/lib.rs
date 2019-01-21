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
    error::RibosomeEncodedValue, error::ZomeApiInternalResult, json::JsonString, json::RawString,
};
use std::convert::TryInto;
use holochain_core_types::entry::Entry;
use holochain_core_types::error::RibosomeRuntimeBits;
use holochain_core_types::error::RibosomeEncodingBits;
use holochain_wasm_utils::memory::stack::WasmStack;
use holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result;
use holochain_wasm_utils::memory::ribosome::load_ribosome_encoded_json;

//-------------------------------------------------------------------------------------------------
// HC DEBUG Function Call
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_debug(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits;
}

/// Call HC API DEBUG function with proper input struct: a string
/// return error code
fn hdk_debug(mem_stack: &mut WasmStack, json_string: &JsonString) {
    // Write input string on stack
    let allocation = match mem_stack.write_json(json_string.to_owned()) {
        Ok(allocation) => allocation,
        Err(_) => return,
    };

    // Call WASMI-able DEBUG
    unsafe { hc_debug(allocation.as_ribosome_encoding()) };

    // Free input allocation and all allocations made inside print()
    mem_stack.deallocate(allocation).expect("deallocate failed");
}

//-------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_hello(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    hdk_debug(
        &mut mem_stack,
        &JsonString::from(RawString::from("Hello world!")),
    );
    RibosomeEncodedValue::Success.into()
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_multiple(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("Hello")));
    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("world")));
    hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("!")));

    RibosomeEncodedValue::Success.into()
}

//-------------------------------------------------------------------------------------------------
//  More tests
//-------------------------------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn debug_stacked_hello(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {
    #[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
    struct TestStruct {
        value: String,
    }

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    let fish = mem_stack.write_json(
        TestStruct {
            value: "fish".to_string(),
        },
    );
    hdk_debug(&mut mem_stack, &JsonString::from("disruptive debug log"));
    return_code_for_allocation_result(
        fish
    ).into()
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Successful
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_commit_entry(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits;
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
    let allocation_of_input = mem_stack.write_json(entry)?;

    // Call WASMI-able commit
    let encoded_allocation_of_result = unsafe { hc_commit_entry(allocation_of_input.as_ribosome_encoding()) };
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_ribosome_encoded_json(encoded_allocation_of_result)?;

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
    let allocation_of_input = mem_stack.write_json(input)?;

    // Call WASMI-able commit
    let encoded_allocation_of_result = unsafe { hc_commit_entry(allocation_of_input.as_ribosome_encoding()) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_ribosome_encoded_json(encoded_allocation_of_result)?;

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

#[derive(Deserialize, Serialize, Default, Debug, DefaultJson)]
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
pub extern "C" fn commit_test(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    let result = hdk_commit(&mut mem_stack, "testEntryType", "hello");

    return_code_for_allocation_result(
        mem_stack.write_json(result)
    ).into()
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn commit_fail_test(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    let result = hdk_commit_fail(&mut mem_stack);

    return_code_for_allocation_result(
        mem_stack.write_json(result)
    ).into()
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(_encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeRuntimeBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    return_code_for_allocation_result(
        mem_stack.write_json(RawString::from("ChainFull"))
    ).into()
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn round_trip_test(encoded_allocation_of_input: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut mem_stack = match WasmStack::try_from_ribosome_encoding(encoded_allocation_of_input) {
        Ok(mem_stack) => mem_stack,
        Err(code) => return code.into(),
    };

    let input = load_ribosome_encoded_json(encoded_allocation_of_input).unwrap();
    let output = test_inner(input);

    return_code_for_allocation_result(mem_stack.write_json(output)).into()
}
