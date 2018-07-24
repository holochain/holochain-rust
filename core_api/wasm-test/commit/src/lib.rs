extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate holochain_core;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

use holochain_core::nucleus::memory::*;

extern {
  fn commit(encoded_allocation_of_input: i32) -> i32;
}



//--------------------------------------------------------------------------------------------------
// Memory Heap
//--------------------------------------------------------------------------------------------------

struct PageStack {
  top: u16,
}

impl PageStack {

  fn new(last_allocation: &MemoryAllocation) -> Self {
    assert!(last_allocation.mem_offset as u32 + last_allocation.mem_len as u32 <= 65535);
    let stack = PageStack { top: last_allocation.mem_offset + last_allocation.mem_len};
    stack
  }
  fn allocate(&mut self, size: u16
  ) -> u16 {
    assert!(self.top as u32 + size as u32 <= 65535);
    let r = self.top;
    self.top += size;
    r
  }

  fn deallocate(&mut self, allocation: &MemoryAllocation) -> Result<(), ()> {
    if self.top == allocation.mem_offset + allocation.mem_len {
      self.top = allocation.mem_offset;
      return Ok(());
    }
    Err(())
  }

} // PageStack


//-------------------------------------------------------------------------------------------------
// HC API funcs
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

/// Commit an entry on source chain and broadcast to dht if entry is public
fn hc_commit(stack: &mut PageStack, entry_type_name: &str, entry_content : &str)
  -> Result<String, &'static str>
{
  // change args to struct & serialize data
  let input = CommitInputStruct {
    entry_type_name: entry_type_name.to_string(),
    entry_content: entry_content.to_string(),
  };
  let allocation_of_input =  serialize(stack, input);

  // Call WASMI-able commit
  let mut encoded_allocation_of_result = 0;
  unsafe {
    encoded_allocation_of_result = commit(allocation_of_input.encode() as i32);
  }
  // Exit if error
//  if encoded_allocation_of_result != 0  {
//    return Ok(encoded_allocation_of_result.to_string())
//  }

  let allocation_of_result = MemoryAllocation::new(encoded_allocation_of_result as u32);


  // Deserialize complex result stored in memory
  // let ptr: *mut c_char =
  let output : CommitOutputStruct = deserialize(allocation_of_result.mem_offset as *mut c_char);

  // FIXME free result & input allocations
  stack.deallocate(&allocation_of_input);

  // Return hash
  Ok(output.hash.to_string())
}


//-------------------------------------------------------------------------------------------------
// Utils
//-------------------------------------------------------------------------------------------------

// Convert json data in a memory buffer into a meaningful data struct
fn deserialize<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> T {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    let actual_str = ptr_safe_c_str.to_str().unwrap();
    serde_json::from_str(actual_str).unwrap()
}


// Write a data struct into a memory buffer as json string
fn serialize<T: Serialize>(stack: &mut PageStack, internal: T) -> MemoryAllocation {
    let json_bytes     = serde_json::to_vec(&internal).unwrap();
    let json_bytes_len = json_bytes.len();
    assert!(json_bytes_len < 65536);

    let ptr = stack.allocate(json_bytes_len as u16) as *mut c_char;

    let ptr_safe  = unsafe { slice::from_raw_parts_mut(ptr, json_bytes_len) };

    for (i, byte) in json_bytes.iter().enumerate() {
      ptr_safe[i] = *byte as i8;
    }

    MemoryAllocation { mem_offset: ptr as u16, mem_len: json_bytes_len as u16 }
}


//-------------------------------------------------------------------------------------------------
// Zome API
//-------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of a memory allocation
/// returns encoded allocation of output
#[no_mangle]
pub extern "C" fn test_dispatch(encoded_allocation_of_input: usize) -> i32 {
  let allocation_of_input = MemoryAllocation::new(encoded_allocation_of_input as u32);
  let mut stack = PageStack::new(&allocation_of_input);

  // let ptr_data_commit = params_len as *mut c_char;
  let output = test(&mut stack);
  let allocation_of_output = serialize(&mut stack, output);
  return allocation_of_output.encode() as i32;
}


/// Actual test function code
fn test(stack: &mut PageStack) -> CommitOutputStruct
{
  // Call Commit API function
  let hash = hc_commit(stack, "post", "hello");

  // Return result in complex format
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
  }
}
