extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

#[allow(unknown_lints)]
#[allow(cast_lossless)]

//--------------------------------------------------------------------------------------------------
// Error Codes
//--------------------------------------------------------------------------------------------------

/// Enumeration of all possible return codes that an HC API function can return
#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum HcApiReturnCode {
    SUCCESS = 0,
    ERROR_SERDE_JSON = 1 << 16,
    ERROR_PAGE_OVERFLOW = 2 << 16,
    ERROR = 3 << 16,
}


//pub fn decode_error(encoded_allocation : u32) -> HcApiReturnCode {
//
//}

pub fn encode_error(offset: u16) -> HcApiReturnCode {
    match offset {
        0 => HcApiReturnCode::SUCCESS,
        1 => HcApiReturnCode::ERROR_SERDE_JSON,
        2 => HcApiReturnCode::ERROR_PAGE_OVERFLOW,
        _ => HcApiReturnCode::ERROR,
    }
}


//--------------------------------------------------------------------------------------------------
// Single Page Memory Allocation
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
/// SinglePageAllocation is a memory allocation garanteed to fit in a WASM 64KiB Memory Page
pub struct SinglePageAllocation {
    pub offset: u16,
    pub length: u16,
}

impl SinglePageAllocation {
    /// An Encoded Allocation is a u32 where 'offset' is first 16-bits and 'length' last 16-bits
    /// A valid allocation must not have a length of zero
    /// An Encoded Allocation with an offset but no length is actually an encoding of an ErrorCode
    pub fn new(encoded_allocation: u32) -> Result<Self, HcApiReturnCode> {
        let allocation = SinglePageAllocation {
            offset: (encoded_allocation >> 16) as u16,
            length: (encoded_allocation % 65536) as u16,
        };
        if allocation.length == 0 {
            return Err(encode_error(allocation.offset));
        }
        if (allocation.offset as u32 + allocation.length as u32) > 65535 {
            return Err(HcApiReturnCode::ERROR_PAGE_OVERFLOW);
        }
        Ok(allocation)
    }

    pub fn encode(self) -> u32 {
        ((self.offset as u32) << 16) + self.length as u32
    }
}


//--------------------------------------------------------------------------------------------------
// Single Page Memory Stack Manager
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Default, Debug)]
/// Struct for managing a WASM 64KiB memory page as a stack
pub struct SinglePageStack {
    top: u16,
}


impl SinglePageStack {
    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new(last_allocation: SinglePageAllocation) -> Self {
        assert!(last_allocation.offset as u32 + last_allocation.length as u32 <= 65535);
        SinglePageStack {
            top: last_allocation.offset + last_allocation.length,
        }
    }

    pub fn new_from_encoded(encoded_last_allocation: u32) -> Self {
        let last_allocation = SinglePageAllocation::new(encoded_last_allocation as u32);
        let last_allocation =
            last_allocation.expect("received error instead of valid encoded allocation");
        assert!(last_allocation.offset as u32 + last_allocation.length as u32 <= 65535);
        return SinglePageStack::new(last_allocation);
    }

    pub fn allocate(&mut self, size: u16) -> u16 {
        assert!(self.top as u32 + size as u32 <= 65535);
        let offset = self.top;
        self.top += size;
        offset
    }

    pub fn deallocate(&mut self, allocation: SinglePageAllocation) -> Result<(), ()> {
        if self.top == allocation.offset + allocation.length {
            self.top = allocation.offset;
            return Ok(());
        }
        Err(())
    }

    // Getters
    pub fn top(self) -> u16 {
        self.top
    }
}


//-------------------------------------------------------------------------------------------------
// Serialization
//-------------------------------------------------------------------------------------------------

// Convert json data in a memory buffer into a meaningful data struct
#[allow(unknown_lints)]
#[allow(not_unsafe_ptr_arg_deref)]
pub fn deserialize<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> T {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    let actual_str = ptr_safe_c_str.to_str().unwrap();
    serde_json::from_str(actual_str).unwrap()
}

// Helper for retrieving struct from encoded allocation
pub fn deserialize_allocation<'s, T: Deserialize<'s>>(encoded_allocation: u32) -> T {
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation.expect("received error instead of valid encoded allocation");
    return deserialize(allocation.offset as *mut c_char);
}

// Helper for retrieving struct or ERROR from encoded allocation
pub fn try_deserialize_allocation<'s, T: Deserialize<'s>>(
    encoded_allocation: u32,
) -> Result<T, HcApiReturnCode> {
    let allocation = SinglePageAllocation::new(encoded_allocation);
    if let Err(e) = allocation {
        return Err(e);
    }
    return Ok(deserialize(allocation.unwrap().offset as *mut c_char));
}

// Write a data struct into a memory buffer as json string
pub fn serialize<T: Serialize>(stack: &mut SinglePageStack, internal: T) -> SinglePageAllocation {
    let json_bytes = serde_json::to_vec(&internal).unwrap();
    let json_bytes_len = json_bytes.len();
    assert!(json_bytes_len < 65536);

    let ptr = stack.allocate(json_bytes_len as u16) as *mut c_char;

    let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, json_bytes_len) };

    for (i, byte) in json_bytes.iter().enumerate() {
        ptr_safe[i] = *byte as i8;
    }

    SinglePageAllocation {
        offset: ptr as u16,
        length: json_bytes_len as u16,
    }
}

// Helper
pub fn serialize_into_encoded_allocation<T: Serialize>(
    stack: &mut SinglePageStack,
    internal: T,
) -> i32 {
    let allocation_of_output = serialize(stack, internal);
    return allocation_of_output.encode() as i32;
}
