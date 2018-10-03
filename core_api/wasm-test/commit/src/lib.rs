extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};

extern {
  fn hc_commit_entry(encoded_allocation_of_input: i32) -> i32;
}


#[derive(Serialize, Default)]
struct CommitInputStruct {
  entry_type_name: String,
  entry_content: String,
}

#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
  hash: String,
}


//-------------------------------------------------------------------------------------------------
// HC Commit Function Call - Succesful
//-------------------------------------------------------------------------------------------------

/// Call HC API COMMIT function with proper input struct
/// return hash of entry added source chain
fn hdk_commit(mem_stack: &mut SinglePageStack, entry_type_name: &str, entry_content: &str)
  -> Result<String, String>
{
  // Put args in struct and serialize into memory
  let input = CommitInputStruct {
    entry_type_name: entry_type_name.to_string(),
    entry_content: entry_content.to_string(),
  };
  let allocation_of_input =  serialize(mem_stack, input);

  // Call WASMI-able commit
  let encoded_allocation_of_result: i32;
  unsafe {
    encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
  }
  // Deserialize complex result stored in memory
  let output: CommitOutputStruct = try_deserialize_allocation(encoded_allocation_of_result as u32)?;

  // Free result & input allocations and all allocations made inside commit()
  mem_stack.deallocate(allocation_of_input).expect("deallocate failed");

  // Return hash
  Ok(output.hash.to_string())
}


//-------------------------------------------------------------------------------------------------
// HC COMMIT Function Call - Fail
//-------------------------------------------------------------------------------------------------

// Simulate error in commit function by inputing output struct as input
fn hdk_commit_fail(mem_stack: &mut SinglePageStack)
  -> Result<String, String>
{
  // Put args in struct and serialize into memory
  let input = CommitOutputStruct {
    hash: "whatever".to_string(),
  };
  let allocation_of_input =  serialize(mem_stack, input);

  // Call WASMI-able commit
  let encoded_allocation_of_result: i32;
  unsafe {
    encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as i32);
  }
  // Deserialize complex result stored in memory
  let output: CommitOutputStruct  = try_deserialize_allocation(encoded_allocation_of_result as u32)?;

  // Free result & input allocations and all allocations made inside commit()
  mem_stack.deallocate(allocation_of_input).expect("deallocate failed");

  // Return hash
  Ok(output.hash.to_string())
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
  let mut mem_stack = SinglePageStack::from_encoded(encoded_allocation_of_input as u32);
  let output = hdk_commit(&mut mem_stack, "testEntryType", "hello");
  return serialize_into_encoded_allocation(&mut mem_stack, output);
}

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn test_fail(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::from_encoded(encoded_allocation_of_input as u32);
  let output = hdk_commit_fail(&mut mem_stack);
  return serialize_into_encoded_allocation(&mut mem_stack, output);
}
