#![feature(try_from)]
extern crate holochain_core_types;
extern crate holochain_wasm_utils;

use holochain_wasm_utils::{
  memory_allocation::*, memory_serialization::*,
};
use holochain_core_types::json::JsonString;
use holochain_core_types::entry::SerializedEntry;
use holochain_core_types::error::ZomeApiInternalResult;
use holochain_core_types::cas::content::Address;
use std::convert::TryInto;
use holochain_core_types::error::RibosomeErrorCode;

extern {
  fn hc_commit_entry(encoded_allocation_of_input: i32) -> i32;
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Successful
//-------------------------------------------------------------------------------------------------

/// Call HC API COMMIT function with proper input struct
/// return hash of entry added source chain
fn hdk_commit(mem_stack: &mut SinglePageStack, entry_type_name: &str, entry_value: &str)
  -> Result<Address, String>
{
  // Put args in struct and serialize into memory
  let serialized_entry = SerializedEntry::new(
    entry_type_name,
    entry_value,
  );
  let allocation_of_input =  store_as_json(mem_stack, JsonString::from(serialized_entry))?;

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
fn hdk_commit_fail(mem_stack: &mut SinglePageStack)
  -> Result<Address, String>
{
  // Put args in struct and serialize into memory
  let input = ZomeApiInternalResult::failure(Address::from("whatever"));
  let allocation_of_input =  store_as_json(mem_stack, input)?;

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

  // if result.ok {
  //     Ok(JsonString::from(result.value).try_into()?)
  // } else {
  //     Err(ZomeApiError::from(result.error))
  // }
  match JsonString::from(result.value).try_into() {
      Ok(address) => Ok(address),
      Err(hc_err) => Err(hc_err.into()),
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
pub extern "C" fn test(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  let result = hdk_commit(&mut mem_stack, "testEntryType", "hello");
  store_as_json_into_encoded_allocation(&mut mem_stack, result)
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn test_fail(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  let result = hdk_commit_fail(&mut mem_stack);
  store_as_json_into_encoded_allocation(&mut mem_stack, result)
}

#[no_mangle]
pub extern fn __hdk_get_validation_package_for_entry_type(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  store_string_into_encoded_allocation(&mut mem_stack, "\"ChainFull\"")
}
