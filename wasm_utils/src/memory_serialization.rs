use crate::memory_allocation::{
    decode_encoded_allocation, SinglePageAllocation, SinglePageStack, U16_MAX,
};
use holochain_core_types::{
    error::{CoreError, HolochainError, RibosomeErrorCode, RibosomeReturnCode},
    json::JsonString,
};
use serde::Deserialize;
use serde_json;
use std::{convert::TryInto, ffi::CStr, os::raw::c_char, slice};

//-------------------------------------------------------------------------------------------------
// Raw
//-------------------------------------------------------------------------------------------------

/// Convert a string stored in wasm memory into a String.
fn load_str_from_raw<'a>(ptr_data: *mut c_char) -> &'a str {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    ptr_safe_c_str.to_str().unwrap()
}

/// Write in wasm memory according to stack state.
fn write_in_wasm_memory(
    stack: &mut SinglePageStack,
    bytes: &[u8],
    len: u16,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    if u32::from(len) + u32::from(stack.top()) > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    let ptr = stack.allocate(len) as *mut c_char;
    let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, len as usize) };
    for (i, byte) in bytes.iter().enumerate() {
        ptr_safe[i] = *byte as i8;
    }
    SinglePageAllocation::new(ptr as u16, len)
}

//-------------------------------------------------------------------------------------------------
// String
//-------------------------------------------------------------------------------------------------

/// Write a string in wasm memory according to stack state.
pub fn store_string(
    stack: &mut SinglePageStack,
    s: &str,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    if len > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }

    write_in_wasm_memory(stack, bytes, len as u16)
}

// Sugar
pub fn store_string_into_encoded_allocation(stack: &mut SinglePageStack, s: &str) -> i32 {
    store_string(stack, s).unwrap().encode() as i32
}

/// Retrieve a stored string from an encoded allocation.
/// Return error code if encoded_allocation is invalid.
pub fn load_string(encoded_allocation: u32) -> Result<String, RibosomeErrorCode> {
    let maybe_allocation = decode_encoded_allocation(encoded_allocation);
    match maybe_allocation {
        Err(return_code) => match return_code {
            RibosomeReturnCode::Success => Err(RibosomeErrorCode::ZeroSizedAllocation),
            RibosomeReturnCode::Failure(err_code) => Err(err_code),
        },
        Ok(allocation) => Ok(load_str_from_raw(allocation.offset() as *mut c_char).to_string()),
    }
}

//-------------------------------------------------------------------------------------------------
// JSON
//-------------------------------------------------------------------------------------------------

/// Write a data struct as a json string in wasm memory according to stack state.
pub fn store_as_json<J: TryInto<JsonString>>(
    stack: &mut SinglePageStack,
    jsonable: J,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    let j: JsonString = jsonable
        .try_into()
        .map_err(|_| RibosomeErrorCode::ArgumentDeserializationFailed)?;

    let json_bytes = j.into_bytes();
    let json_bytes_len = json_bytes.len() as u32;
    if json_bytes_len > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    write_in_wasm_memory(stack, &json_bytes, json_bytes_len as u16)
}

// Sugar
pub fn store_as_json_into_encoded_allocation<J: TryInto<JsonString>>(
    stack: &mut SinglePageStack,
    jsonable: J,
) -> i32 {
    store_as_json(stack, jsonable).unwrap().encode() as i32
}

/// Retrieve a stored data struct from an encoded allocation.
/// Return error string if encoded_allocation is invalid.
pub fn load_json<'s, T: Deserialize<'s>>(encoded_allocation: u32) -> Result<T, HolochainError> {
    let maybe_allocation = decode_encoded_allocation(encoded_allocation);
    match maybe_allocation {
        Err(return_code) => match return_code {
            RibosomeReturnCode::Success => Err(HolochainError::Ribosome(
                RibosomeErrorCode::ZeroSizedAllocation,
            )),
            RibosomeReturnCode::Failure(err_code) => Err(HolochainError::Ribosome(err_code)),
        },
        Ok(allocation) => load_json_from_raw(allocation.offset() as *mut c_char),
    }
}

/// Convert a json string stored in wasm memory into a specified struct
/// If json deserialization of custom struct failed, tries to deserialize a CoreError struct.
/// If that also failed, tries to load a string directly, since we are expecting an error string at this stage.
#[allow(unknown_lints)]
pub fn load_json_from_raw<'s, T: Deserialize<'s>>(
    ptr_data: *mut c_char,
) -> Result<T, HolochainError> {
    let stored_str = load_str_from_raw(ptr_data);
    let maybe_obj: Result<T, serde_json::Error> = serde_json::from_str(stored_str);
    match maybe_obj {
        Ok(obj) => Ok(obj),
        Err(_) => {
            // TODO #394 - In Release, load error_string directly and not a CoreError
            let maybe_hc_err: Result<CoreError, serde_json::Error> =
                serde_json::from_str(stored_str);

            Err(match maybe_hc_err {
                Err(_) => {
                    HolochainError::Ribosome(RibosomeErrorCode::ArgumentDeserializationFailed)
                }
                Ok(hc_err) => hc_err.kind,
            })
        }
    }
}
