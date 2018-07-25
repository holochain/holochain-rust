extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;

use holochain_wasm_utils::*;

extern {
  fn commit(encoded_allocation_of_input: i32) -> i32;
}

//-------------------------------------------------------------------------------------------------
// HC Commit Function Call
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Default)]
struct CommitInputStruct {
  entry_type_name: String,
  entry_content: String,
}

#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
  hash: String,
}

/// Call HC API commit function with proper input struct
/// return hash of entry added source chain
fn hc_commit(mem_stack: &mut SinglePageStack, entry_type_name: &str, entry_content : &str)
  -> Result<String, &'static str>
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
    encoded_allocation_of_result = commit(allocation_of_input.encode() as i32);
  }
  // Exit if error
//  if encoded_allocation_of_result != 0  {
//    return Ok(encoded_allocation_of_result.to_string())
//  }


  // Deserialize complex result stored in memory
  let output : CommitOutputStruct = deserialize_allocation(encoded_allocation_of_result as u32);

  // Free result & input allocations
  mem_stack.deallocate(&allocation_of_input).expect("deallocate failed");

  // Return hash
  Ok(output.hash.to_string())
}


/// Actual test function code
fn test(mem_stack: &mut SinglePageStack) -> CommitOutputStruct
{
  // Call Commit API function
  let hash = hc_commit(mem_stack, "post", "hello");

  // Return result in complex format
  return
    if let Ok(hash_str) = hash {
      CommitOutputStruct {
        hash: hash_str,
      }
    }
      else
      {
        CommitOutputStruct {
          hash: "fail".to_string(),
        }
      };
}

//-------------------------------------------------------------------------------------------------
//  Generatable Dispatch function
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn test_dispatch(encoded_allocation_of_input: usize) -> i32 {
  let mut mem_stack = SinglePageStack::new_from_encoded(encoded_allocation_of_input as u32);
  let output = test(&mut mem_stack);
  return serialize_into_encoded_allocation(&mut mem_stack, output);
}


