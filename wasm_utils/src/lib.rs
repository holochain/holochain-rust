
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};


//--------------------------------------------------------------------------------------------------
// Single Page Memory Allocation
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct SinglePageAllocation {
  pub offset: u16,
  pub length: u16,
}


impl SinglePageAllocation {
  pub fn new(input : u32) -> Self {
    let allocation = SinglePageAllocation {
      offset: (input >> 16) as u16,
      length: (input % 65536) as u16,
    };
    assert!(allocation.length > 0);
    assert!((allocation.offset as u32 + allocation.length as u32) <= 65535);
    allocation
  }

  pub fn encode(&self) -> u32 {
    ((self.offset as u32) << 16) + self.length as u32
  }

}


//--------------------------------------------------------------------------------------------------
// Single Page Memory Stack Manager
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Default, Debug)]
pub struct SinglePageStack {
  top: u16,
}


impl SinglePageStack {

  pub fn new_from_encoded(encoded_last_allocation: u32) -> Self {
    let last_allocation = SinglePageAllocation::new(encoded_last_allocation as u32);
    assert!(last_allocation.offset as u32 + last_allocation.length as u32 <= 65535);
    return SinglePageStack::new(&last_allocation);
  }

  pub fn new(last_allocation: &SinglePageAllocation) -> Self {
    assert!(last_allocation.offset as u32 + last_allocation.length as u32 <= 65535);
    let stack = SinglePageStack { top: last_allocation.offset + last_allocation.length };
    stack
  }

  pub fn allocate(&mut self, size: u16) -> u16 {
    assert!(self.top as u32 + size as u32 <= 65535);
    let offset = self.top;
    self.top += size;
    offset
  }

  pub fn deallocate(&mut self, allocation: &SinglePageAllocation) -> Result<(), ()> {
    if self.top == allocation.offset + allocation.length {
      self.top = allocation.offset;
      return Ok(());
    }
    Err(())
  }

  // Getters
  pub fn top(&self) -> u16 { self.top }
}


//-------------------------------------------------------------------------------------------------
// Serialization
//-------------------------------------------------------------------------------------------------

// Convert json data in a memory buffer into a meaningful data struct
pub fn deserialize<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> T {
  let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
  let actual_str = ptr_safe_c_str.to_str().unwrap();
  serde_json::from_str(actual_str).unwrap()
}


// Helper
pub fn deserialize_allocation<'s, T: Deserialize<'s>>(encoded_allocation: u32) -> T {
  let allocation = SinglePageAllocation::new(encoded_allocation);
  return deserialize(allocation.offset as *mut c_char);
}


// Write a data struct into a memory buffer as json string
pub fn serialize<T: Serialize>(stack: &mut SinglePageStack, internal: T) -> SinglePageAllocation {
  let json_bytes     = serde_json::to_vec(&internal).unwrap();
  let json_bytes_len = json_bytes.len();
  assert!(json_bytes_len < 65536);

  let ptr = stack.allocate(json_bytes_len as u16) as *mut c_char;

  let ptr_safe  = unsafe { slice::from_raw_parts_mut(ptr, json_bytes_len) };

  for (i, byte) in json_bytes.iter().enumerate() {
    ptr_safe[i] = *byte as i8;
  }

  SinglePageAllocation { offset: ptr as u16, length: json_bytes_len as u16 }
}

// Helper
pub fn serialize_into_encoded_allocation<T: Serialize>(stack: &mut SinglePageStack, internal: T) -> i32 {
  let allocation_of_output = serialize(stack, internal);
  return allocation_of_output.encode() as i32;
}