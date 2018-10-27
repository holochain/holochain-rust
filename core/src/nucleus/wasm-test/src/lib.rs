extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};

extern {
  fn hc_debug(encoded_allocation_of_input: i32) -> i32;
}

//-------------------------------------------------------------------------------------------------
// HC DEBUG Function Call
//-------------------------------------------------------------------------------------------------

/// Call HC API DEBUG function with proper input struct: a string
/// return error code
fn hdk_debug(mem_stack: &mut SinglePageStack, s: &str) {
  // Write input string on stack
  let maybe_allocation =  store_as_json(mem_stack, s);
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
  hdk_debug(&mut mem_stack, "Hello world!");
  return 0;
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
#[no_mangle]
pub extern "C" fn debug_multiple(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  hdk_debug(&mut mem_stack, "Hello");
  hdk_debug(&mut mem_stack, "world");
  hdk_debug(&mut mem_stack, "!");
  return 0;
}

//-------------------------------------------------------------------------------------------------
//  More tests
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Default, Clone, PartialEq, Deserialize)]
struct TestStruct {
  value: String,
}

#[no_mangle]
pub extern "C" fn debug_stacked_hello(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded_allocation(encoded_allocation_of_input as u32).unwrap();
  let fish = store_json_into_encoded_allocation(&mut mem_stack, TestStruct {
    value: "fish".to_string(),
  });
  hdk_debug(&mut mem_stack, "disruptive debug log");
  fish
}