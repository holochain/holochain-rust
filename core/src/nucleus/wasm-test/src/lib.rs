extern crate holochain_wasm_utils;
use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::json::RawString;
use holochain_wasm_utils::holochain_core_types::error::RibosomeReturnCode;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};

extern {
  fn hc_debug(encoded_allocation_of_input: i32) -> i32;
}

//-------------------------------------------------------------------------------------------------
// HC DEBUG Function Call
//-------------------------------------------------------------------------------------------------

/// Call HC API DEBUG function with proper input struct: a string
/// return error code
fn hdk_debug(mem_stack: &mut SinglePageStack, json_string: &JsonString) {
  // Write input string on stack
  let maybe_allocation =  store_json(mem_stack, json_string.to_owned());
  if let Err(_) = maybe_allocation {
    return;
  }
  let allocation_of_input = maybe_allocation.unwrap();
  // Call WASMI-able DEBUG
  unsafe {
    hc_debug(allocation_of_input.encode() as i32);
  }
  // Free input allocation and all allocations made inside print()
  mem_stack.deallocate(allocation_of_input).expect("deallocate failed");
}


//-------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_hello(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("Hello world!")));
  i32::from(RibosomeReturnCode::Success)
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_multiple(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("Hello")));
  hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("world")));
  hdk_debug(&mut mem_stack, &JsonString::from(RawString::from("!")));
  i32::from(RibosomeReturnCode::Success)
}
