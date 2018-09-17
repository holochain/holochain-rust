extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

//--------------------------------------------------------------------------------------------------
// Error Codes
//--------------------------------------------------------------------------------------------------

/// Enumeration of all possible return codes that an HC API function can return
/// represents a zero length offset in SinglePageAllocation
/// @see SinglePageAllocation
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum HcApiReturnCode {
    Success = 0,
    Error = 1 << 16,
    ErrorJson = 2 << 16,
    ErrorPageOverflow = 3 << 16,
    ErrorActionResult = 4 << 16,
    ErrorCallbackResult = 5 << 16,
    ErrorRecursiveCall = 6 << 16,
}

//pub fn decode_error(encoded_allocation: u32) -> HcApiReturnCode {
//
//}

pub fn encode_error(offset: u16) -> HcApiReturnCode {
    match offset {
        // @TODO what is a success error?
        // @see https://github.com/holochain/holochain-rust/issues/181
        0 => HcApiReturnCode::Success,
        2 => HcApiReturnCode::ErrorJson,
        3 => HcApiReturnCode::ErrorPageOverflow,
        4 => HcApiReturnCode::ErrorActionResult,
        5 => HcApiReturnCode::ErrorCallbackResult,
        6 => HcApiReturnCode::ErrorRecursiveCall,
        1 | _ => HcApiReturnCode::Error,
    }
}

/// returns the u16 high bits from a u32
pub fn u32_high_bits(i: u32) -> u16 {
    (i >> 16) as u16
}

/// returns the u16 low bits from a u32
pub fn u32_low_bits(i: u32) -> u16 {
    (i as u16 % std::u16::MAX)
}

/// splits the high and low bits of u32 into a tuple of u16, for destructuring convenience
pub fn u32_split_bits(i: u32) -> (u16, u16) {
    (u32_high_bits(i), u32_low_bits(i))
}

/// merges 2x u16 into a single u32
pub fn u32_merge_bits(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) | u32::from(low)
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

#[allow(unknown_lints)]
#[allow(cast_lossless)]
impl SinglePageAllocation {
    /// An Encoded Allocation is a u32 where 'offset' is first 16-bits and 'length' last 16-bits
    /// A valid allocation must not have a length of zero
    /// An Encoded Allocation with an offset but no length is actually an encoding of an ErrorCode
    pub fn new(encoded_allocation: u32) -> Result<Self, HcApiReturnCode> {
        let (offset, length) = u32_split_bits(encoded_allocation);
        let allocation = SinglePageAllocation { offset, length };

        // zero length allocation = encoding an error api return code
        if allocation.length == 0 {
            // @TODO is it right to return success as Err for 0? what is a "success" error?
            // @see https://github.com/holochain/holochain-rust/issues/181
            return Err(encode_error(allocation.offset));
        }

        // should never happen
        // we don't panic because this needs to work with wasm, which doesn't support panic
        if (allocation.offset as u32 + allocation.length as u32) > std::u16::MAX as u32 {
            return Err(HcApiReturnCode::ErrorPageOverflow);
        }

        Ok(allocation)
    }

    /// returns a single u32 value encoding both the u16 offset and length values
    pub fn encode(self) -> u32 {
        u32_merge_bits(self.offset, self.length)
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

#[allow(unknown_lints)]
#[allow(cast_lossless)]
impl SinglePageStack {
    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new(last_allocation: SinglePageAllocation) -> Self {
        assert!(
            last_allocation.offset as u32 + last_allocation.length as u32 <= std::u16::MAX as u32
        );
        SinglePageStack {
            top: last_allocation.offset + last_allocation.length,
        }
    }

    pub fn from_encoded(encoded_last_allocation: u32) -> Self {
        let last_allocation = SinglePageAllocation::new(encoded_last_allocation as u32);
        let last_allocation =
            last_allocation.expect("received error instead of valid encoded allocation");
        assert!(
            last_allocation.offset as u32 + last_allocation.length as u32 <= std::u16::MAX as u32
        );
        return SinglePageStack::new(last_allocation);
    }

    pub fn allocate(&mut self, size: u16) -> u16 {
        assert!(self.top as u32 + size as u32 <= std::u16::MAX as u32);
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
    assert!(json_bytes_len < std::u16::MAX as usize);

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

#[cfg(test)]
pub mod tests {

    use super::{HcApiReturnCode, SinglePageAllocation};

    #[test]
    /// tests that encoding integers for errors returns the correct return code
    fn encode_error() {
        assert_eq!(super::encode_error(0), HcApiReturnCode::Success);

        assert_eq!(super::encode_error(1), HcApiReturnCode::Error);

        assert_eq!(super::encode_error(2), HcApiReturnCode::ErrorJson);

        assert_eq!(super::encode_error(3), HcApiReturnCode::ErrorPageOverflow);

        assert_eq!(super::encode_error(4), HcApiReturnCode::ErrorActionResult);

        assert_eq!(super::encode_error(5), HcApiReturnCode::ErrorCallbackResult);

        assert_eq!(super::encode_error(6), HcApiReturnCode::ErrorRecursiveCall);

        assert_eq!(super::encode_error(7), HcApiReturnCode::Error);
    }

    #[test]
    /// tests construction and encoding in a new single page allocation
    fn new_spa() {
        let i = 0b1010101010101010_0101010101010101;
        let spa = SinglePageAllocation::new(i).unwrap();

        assert_eq!(0b1010101010101010, spa.offset);

        assert_eq!(0b0101010101010101, spa.length);
    }

    #[test]
    /// tests that we can encode error return codes (zero length allocation)
    fn new_spa_error() {
        assert_eq!(
            // offset 0 = success?
            // @see https://github.com/holochain/holochain-rust/issues/181
            SinglePageAllocation::new(0b0000000000000000_0000000000000000).unwrap_err(),
            HcApiReturnCode::Success,
        );

        assert_eq!(
            // offset 1 = generic error
            SinglePageAllocation::new(0b0000000000000001_0000000000000000).unwrap_err(),
            HcApiReturnCode::Error,
        );

        assert_eq!(
            // offset 2 = serde json error
            SinglePageAllocation::new(0b0000000000000010_0000000000000000).unwrap_err(),
            HcApiReturnCode::ErrorJson,
        );

        assert_eq!(
            // offset 3 = page overflow error
            SinglePageAllocation::new(0b0000000000000011_0000000000000000).unwrap_err(),
            HcApiReturnCode::ErrorPageOverflow,
        );

        assert_eq!(
            // offset 4 = page overflow error
            SinglePageAllocation::new(0b0000000000000100_0000000000000000).unwrap_err(),
            HcApiReturnCode::ErrorActionResult,
        );

        assert_eq!(
            // nonsense offset = generic error
            SinglePageAllocation::new(0b1010101010101010_0000000000000000).unwrap_err(),
            HcApiReturnCode::Error,
        );
    }

    #[test]
    /// tests that a SinglePageAllocation returns its encoded offset/length pair as u32
    fn spa_encode() {
        let i = 0b1010101010101010_0101010101010101;
        let spa = SinglePageAllocation::new(i).unwrap();

        assert_eq!(i, spa.encode());
    }

    #[test]
    /// tests that we can extract the high bits from a u32 into the correct u16
    fn u32_high_bits() {
        assert_eq!(
            0b1010101010101010,
            super::u32_high_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    /// tests that we can extract the high bits from a u32 into the correct u16
    fn u32_low_bits() {
        assert_eq!(
            0b0101010101010101,
            super::u32_low_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    /// tests that we can split a u32 into a tuple of high/low bits
    fn u32_split_bits() {
        assert_eq!(
            (0b1010101010101010, 0b0101010101010101),
            super::u32_split_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    /// tests that we can merge a u16 tuple into a u32
    fn u32_merge_bits() {
        assert_eq!(
            0b1010101010101010_0101010101010101,
            super::u32_merge_bits(0b1010101010101010, 0b0101010101010101),
        );
    }

}
