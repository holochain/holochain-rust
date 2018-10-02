
use serde_json;
use error::RibosomeErrorReport;
use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};
use memory_allocation::{SinglePageStack, SinglePageAllocation};

// Convert a json string stored in wasm memory into a specified struct
// If json deserialization of custom struct failed, tries to deserialize a RibosomeErrorReport struct.
// If that also failed, tries to load a string directly, since we are expecting an error string at this stage.
#[allow(unknown_lints)]
#[allow(not_unsafe_ptr_arg_deref)]
pub fn deserialize<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> Result<T, String> {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    let actual_str = ptr_safe_c_str.to_str().unwrap();
    let res = serde_json::from_str(actual_str);
    match res {
        Err(_) => {
            // TODO #394 - In Release, load error_string directly and not a RibosomeErrorReport
            let maybe_error_report: Result<RibosomeErrorReport, serde_json::Error> =
                serde_json::from_str(actual_str);
            match maybe_error_report {
                Err(_) => Err(actual_str.to_string()),
                Ok(error_report) => Err(error_report.description),
            }
        }
        Ok(x) => Ok(x),
    }
}

// Helper for retrieving struct from encoded allocation
pub fn deserialize_allocation<'s, T: Deserialize<'s>>(encoded_allocation: u32) -> T {
    let allocation = SinglePageAllocation::new(encoded_allocation);
    let allocation = allocation.expect("received error instead of valid encoded allocation");
    return deserialize(allocation.offset as *mut c_char).unwrap();
}

// Helper for retrieving struct or ERROR from encoded allocation
pub fn try_deserialize_allocation<'s, T: Deserialize<'s>>(
    encoded_allocation: u32,
) -> Result<T, String> {
    let maybe_allocation = SinglePageAllocation::new(encoded_allocation);
    match maybe_allocation {
        Err(return_code) => Err(return_code.to_string()),
        Ok(allocation) => deserialize(allocation.offset as *mut c_char),
    }
}

// Write a data struct into a memory buffer as json string
pub fn serialize<T: Serialize>(stack: &mut SinglePageStack, internal: T) -> SinglePageAllocation {
    let json_bytes = serde_json::to_vec(&internal).unwrap();
    let json_bytes_len = json_bytes.len();
    assert!(json_bytes_len < <u16>::max_value() as usize);

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
