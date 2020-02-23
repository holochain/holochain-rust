extern crate holochain_core_types;
#[macro_use]
extern crate holochain_json_derive;
#[macro_use]
extern crate serde_derive;
extern crate holochain_wasm_types;
extern crate serde_json;
extern crate holochain_wasmer_guest;
extern crate holochain_persistence_api;

use holochain_core_types::{
    entry::Entry,
    error::{
        ZomeApiInternalResult,
    },
    signature::Provenance,
    validation::{ValidationPackageDefinition, ValidationResult},
};

use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::{
    holochain_json_api::{error::JsonError, json::{JsonString, RawString}},
};
use holochain_wasmer_guest::*;

use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// HC DEBUG Function Call
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_debug(host_allocation_ptr: AllocationPtr) -> AllocationPtr;
}

//-------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// host_allocation_ptr : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_hello(
    _: AllocationPtr,
) -> AllocationPtr {
    let _: () = try_result!(host_call!(hc_debug, RawString::from("Hello world!")), "failed to handle hc_debug result");
    ret!(());
}

/// Function called by Holochain Instance
/// host_allocation_ptr : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_multiple(
    _: AllocationPtr,
) -> AllocationPtr {
    let _: () = try_result!(host_call!(hc_debug, RawString::from("Hello")), "debug_multiple one");
    let _: () = try_result!(host_call!(hc_debug, RawString::from("world")), "debug_multiple two");
    let _: () = try_result!(host_call!(hc_debug, RawString::from("!")), "debug_multiple three");

    ret!(());
}

//-------------------------------------------------------------------------------------------------
//  More tests
//-------------------------------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn debug_stacked_hello(
    _: AllocationPtr,
) -> AllocationPtr {
    #[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
    struct TestStruct {
        value: String,
    }

    let fish = TestStruct {
        value: "fish".to_string(),
    };
    let _: () = try_result!(host_call!(hc_debug, RawString::from("disruptive debug log")), "debug_stacked_hello fail");
    ret!(fish);
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Successful
//-------------------------------------------------------------------------------------------------

extern "C" {
    fn hc_commit_entry(host_allocation_ptr: AllocationPtr) -> AllocationPtr;
}

/// Call HC API COMMIT function with proper input struct
/// return address of entry added source chain
fn hdk_commit(
    entry_type_name: &str,
    entry_value: &'static str,
    provenance: &Vec<Provenance>
) -> Result<Address, String> {
    // Put args in struct and serialize into memory
    let entry = Entry::App(
        entry_type_name.to_owned().into(),
        RawString::from(entry_value).into(),
    );

    let args = holochain_wasm_types::commit_entry::CommitEntryArgs {
        entry,
        options:holochain_wasm_types::commit_entry::CommitEntryOptions::new(provenance.to_vec())
    };

    let result: ZomeApiInternalResult = host_call!(hc_commit_entry, args)?;
    match JsonString::from_json(&result.value).try_into() {
        Ok(address) => Ok(address),
        Err(hc_err) => Err(hc_err.into()),
    }
}

//-------------------------------------------------------------------------------------------------
// HC COMMIT Function Call - Fail
//-------------------------------------------------------------------------------------------------

// Simulate error in commit function by inputing output struct as input
fn hdk_commit_fail() -> Result<Address, String> {
    // Put args in struct and serialize into memory
    let input = ZomeApiInternalResult::failure(Address::from("whatever"));

    let result: ZomeApiInternalResult = host_call!(hc_commit_entry, input)?;

    let address = JsonString::from_json(&result.value).try_into()?;

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
/// host_allocation_ptr : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn commit_test(
    _: AllocationPtr,
) -> AllocationPtr {
    let result = try_result!(hdk_commit("testEntryType", "hello", &vec![]), "failed to commit in commit_test");
    ret!(result);
}

/// Function called by Holochain Instance
/// host_allocation_ptr : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn commit_fail_test(
    _: AllocationPtr,
) -> AllocationPtr {
    let result = try_result!(hdk_commit_fail(), "failed to fail in commit_fail_test");
    ret!(result);
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(
    _: AllocationPtr,
) -> AllocationPtr {
    ret!(ValidationResult::Ok(()));
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    _: AllocationPtr,
) -> AllocationPtr {
    ret!(ValidationPackageDefinition::ChainFull);
}

#[no_mangle]
pub extern "C" fn round_trip_test(
    host_allocation_ptr: AllocationPtr,
) -> AllocationPtr {
    ret!(test_inner(host_args!(host_allocation_ptr)));
}
