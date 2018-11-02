extern crate holochain_core_types;
extern crate holochain_wasm_utils;

use holochain_core_types::hash::HashString;
use holochain_wasm_utils::{
  api_serialization::commit::{CommitEntryArgs, CommitEntryResult},
  memory_allocation::*, memory_serialization::*,
  holochain_core_types::error::HolochainError,
};

extern {
  fn hc_commit_entry(encoded_allocation_of_input: i32) -> i32;
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Succesful
//-------------------------------------------------------------------------------------------------

/// Call HC API COMMIT function with proper input struct
/// return hash of entry added source chain
fn hdk_commit(mem_stack: &mut SinglePageStack, entry_type_name: &str, entry_value: &str)
  -> Result<String, String>
{
  // Put args in struct and serialize into memory
  let input = CommitEntryArgs {
    entry_type_name: entry_type_name.to_owned(),
    entry_value: entry_value.to_owned(),
  };
  let maybe_allocation =  store_as_json(mem_stack, input);
  if let Err(return_code) = maybe_allocation {
    return Err(return_code.to_string());
  }
  let allocation_of_input = maybe_allocation.unwrap();

  // Call WASMI-able commit
  let encoded_allocation_of_result: i32;
  unsafe {
    encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
  }
  // Deserialize complex result stored in memory
  let result: Result<CommitEntryResult, HolochainError> = load_json(encoded_allocation_of_result as u32);
  // Free result & input allocations
  mem_stack.deallocate(allocation_of_input).expect("deallocate failed");
  // Done
  result
      .map(|entry| entry.address.to_string())
      .map_err(|hc_err| hc_err.to_string())
}


//-------------------------------------------------------------------------------------------------
// HC COMMIT Function Call - Fail
//-------------------------------------------------------------------------------------------------

// Simulate error in commit function by inputing output struct as input
fn hdk_commit_fail(mem_stack: &mut SinglePageStack)
  -> Result<String, String>
{
  // Put args in struct and serialize into memory
  let input = CommitEntryResult {
    address: HashString::from("whatever"),
  };
  let maybe_allocation =  store_as_json(mem_stack, input);
  if let Err(return_code) = maybe_allocation {
    return Err(return_code.to_string());
  }
  let allocation_of_input = maybe_allocation.unwrap();

  // Call WASMI-able commit
  let encoded_allocation_of_result: i32;
  unsafe {
    encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
  }
  // Deserialize complex result stored in memory
  let result: Result<CommitEntryResult, HolochainError> = load_json(encoded_allocation_of_result as u32);
  // Free result & input allocations
  mem_stack.deallocate(allocation_of_input).expect("deallocate failed");
  // Done
  result
      .map(|entry| entry.address.to_string())
      .map_err(|hc_err| hc_err.to_string())
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
  let output = hdk_commit(&mut mem_stack, "testEntryType", "hello");
  return store_json_into_encoded_allocation(&mut mem_stack, output);
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn test_fail(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  let output = hdk_commit_fail(&mut mem_stack);
  return store_json_into_encoded_allocation(&mut mem_stack, output);
}

#[no_mangle]
pub extern fn __hdk_get_validation_package_for_entry_type(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  store_string_into_encoded_allocation(&mut mem_stack, "\"ChainFull\"")
}
