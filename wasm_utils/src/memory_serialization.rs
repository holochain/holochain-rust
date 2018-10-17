use error::{RibosomeErrorCode, RibosomeErrorReport};
use memory_allocation::{
    decode_encoded_allocation, SinglePageAllocation, SinglePageStack, U16_MAX,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{ffi::CStr, os::raw::c_char, slice};

// TODO #486 - load and store string from wasm memory
//pub fn load_string(encoded_allocation: u32) -> Result<String, String> {
//    let maybe_allocation = decode_encoded_allocation(encoded_allocation);
//    match maybe_allocation {
//        Err(return_code) => Err(return_code.to_string()),
//        Ok(allocation) => Ok(load_string_from_allocation(&allocation)),
//    }
//}
///// Write a string in wasm memory
//pub fn store_string(stack: &mut SinglePageStack, s: &str)
//    -> Result<SinglePageAllocation, RibosomeErrorCode> {
//    let mut bytes: Vec<_> = s.to_string().into_bytes();
//    bytes.push(0); // Add string terminate character (important)
//    let len = bytes.len();
//    if len > <u16>::max_value() as usize {
//        return Err(RibosomeErrorCode::OutOfMemory);
//    }
//    return write_in_wasm_memory(stack, &bytes, len as u16);
//}

/// Expecting to retrieve a struct from an encoded allocation, but return error string in case of error
pub fn load_json<'s, T: Deserialize<'s>>(encoded_allocation: u32) -> Result<T, String> {
    let maybe_allocation = decode_encoded_allocation(encoded_allocation);
    match maybe_allocation {
        Err(return_code) => Err(return_code.to_string()),
        Ok(allocation) => load_json_from_raw(allocation.offset() as *mut c_char),
    }
}

/// Write a data struct as a json string in wasm memory
pub fn store_as_json<T: Serialize>(
    stack: &mut SinglePageStack,
    internal: T,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    let json_bytes = serde_json::to_vec(&internal).unwrap();
    let json_bytes_len = json_bytes.len();
    if json_bytes_len > <u16>::max_value() as usize {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    return write_in_wasm_memory(stack, &json_bytes, json_bytes_len as u16);
}

// Sugar
pub fn store_json_into_encoded_allocation<T: Serialize>(
    stack: &mut SinglePageStack,
    internal: T,
) -> i32 {
    let allocation_of_output = store_as_json(stack, internal).unwrap();
    return allocation_of_output.encode() as i32;
}

//-------------------------------------------------------------------------------------------------
// Helper
//-------------------------------------------------------------------------------------------------

/// Convert a json string stored in wasm memory into a specified struct
/// If json deserialization of custom struct failed, tries to deserialize a RibosomeErrorReport struct.
/// If that also failed, tries to load a string directly, since we are expecting an error string at this stage.
#[allow(unknown_lints)]
#[allow(not_unsafe_ptr_arg_deref)]
pub fn load_json_from_raw<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> Result<T, String> {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    let actual_str = ptr_safe_c_str.to_str().unwrap();
    let maybe_obj: Result<T, serde_json::Error> = serde_json::from_str(actual_str);
    match maybe_obj {
        Ok(obj) => Ok(obj),
        Err(_) => {
            // TODO #394 - In Release, load error_string directly and not a RibosomeErrorReport
            let maybe_error_report: Result<RibosomeErrorReport, serde_json::Error> =
                serde_json::from_str(actual_str);
            match maybe_error_report {
                Err(_) => Err(RibosomeErrorCode::ArgumentDeserializationFailed.to_string()),
                Ok(error_report) => Err(error_report.description),
            }
        }
    }
}

// TODO #486 - load and store string from wasm memory
//// Convert a string stored in wasm memory into a String
//fn load_string_from_allocation(alloc: &SinglePageAllocation) -> String {
//    unsafe{
//        String::from_raw_parts(
//            alloc.offset() as *mut u8,
//            alloc.length() as usize,
//            alloc.length() as usize,
//        )
//    }
//}

/// Write in wasm memory according to stack state
fn write_in_wasm_memory(
    stack: &mut SinglePageStack,
    bytes: &Vec<u8>,
    len: u16,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    if len as u32 + stack.top() as u32 > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    let ptr = stack.allocate(len) as *mut c_char;
    let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, len as usize) };
    for (i, byte) in bytes.iter().enumerate() {
        ptr_safe[i] = *byte as i8;
    }
    SinglePageAllocation::new(ptr as u16, len)
}
