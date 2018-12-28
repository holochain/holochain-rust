use holochain_core_types::error::{RibosomeErrorCode, RibosomeReturnCode};

//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

pub const U16_MAX: u32 = <u16>::max_value() as u32;

/// returns the u16 high bits from a u32
pub fn u32_high_bits(i: u32) -> u16 {
    (i >> 16) as u16
}

/// returns the u16 low bits from a u32 by doing a lossy cast
pub fn u32_low_bits(i: u32) -> u16 {
    (i as u16)
}

/// splits the high and low bits of u32 into a tuple of u16, for destructuring convenience
pub fn u32_split_bits(i: u32) -> (u16, u16) {
    (u32_high_bits(i), u32_low_bits(i))
}

/// merges 2x u16 into a single u32
pub fn u32_merge_bits(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) | u32::from(low)
}

pub fn decode_encoded_allocation(
    encoded_allocation: u32,
) -> Result<SinglePageAllocation, RibosomeReturnCode> {
    let (offset, length) = u32_split_bits(encoded_allocation);
    // zero length allocation = RibosomeReturnCode
    if length == 0 {
        return Err(RibosomeReturnCode::from_offset(offset));
    }
    let res = SinglePageAllocation::new(offset, length);
    match res {
        Ok(alloc) => Ok(alloc),
        Err(err_code) => Err(RibosomeReturnCode::Failure(err_code)),
    }
}

//--------------------------------------------------------------------------------------------------
// Single Page Memory Allocation
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
/// SinglePageAllocation is a memory allocation garanteed to fit in a WASM 64KiB Memory Page
pub struct SinglePageAllocation {
    offset: u16,
    length: u16,
}

#[allow(unknown_lints)]
impl SinglePageAllocation {
    pub fn new(offset: u16, length: u16) -> Result<Self, RibosomeErrorCode> {
        if (offset as u32 + length as u32) > U16_MAX {
            return Err(RibosomeErrorCode::OutOfMemory);
        }
        if (offset + length) == 0 {
            return Err(RibosomeErrorCode::ZeroSizedAllocation);
        }
        if length == 0 {
            return Err(RibosomeErrorCode::NotAnAllocation);
        }
        Ok(SinglePageAllocation { offset, length })
    }

    /// An Encoded Allocation is a u32 where 'offset' is first 16-bits and 'length' last 16-bits
    /// A valid allocation must not have a length of zero
    /// An Encoded Allocation with an offset but no length is actually an encoding of an ErrorCode
    pub fn from_encoded_allocation(encoded_allocation: u32) -> Result<Self, RibosomeErrorCode> {
        let maybe_allocation = decode_encoded_allocation(encoded_allocation);
        match maybe_allocation {
            Err(_) => Err(RibosomeErrorCode::NotAnAllocation),
            Ok(allocation) => Ok(allocation),
        }
    }

    /// returns a single u32 value encoding both the u16 offset and length values
    pub fn encode(self) -> u32 {
        u32_merge_bits(self.offset, self.length)
    }

    // getters
    pub fn offset(self) -> u16 {
        self.offset
    }
    pub fn length(self) -> u16 {
        self.length
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
        assert!(u32::from(last_allocation.offset) + u32::from(last_allocation.length) <= U16_MAX);
        SinglePageStack {
            top: last_allocation.offset + last_allocation.length,
        }
    }

    /// Create a SinglePageStack from a valid encoded allocation
    pub fn from_encoded_allocation(
        encoded_last_allocation: u32,
    ) -> Result<Self, RibosomeErrorCode> {
        decode_encoded_allocation(encoded_last_allocation as u32)
            .map(SinglePageStack::new)
            .map_err(|_| RibosomeErrorCode::NotAnAllocation)
    }

    pub fn allocate(&mut self, size: u16) -> u16 {
        assert!(self.top as u32 + size as u32 <= U16_MAX);
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

#[cfg(test)]
pub mod tests {

    use super::*;
    use holochain_core_types::error::RibosomeReturnCode;

    pub fn test_single_page_allocation() -> SinglePageAllocation {
        SinglePageAllocation::new(0, 20).expect("could not create test SinglePageAllocation")
    }

    pub fn test_single_page_stack() -> SinglePageStack {
        SinglePageStack::new(test_single_page_allocation())
    }

    #[test]
    /// smoke test single_page_allocation
    fn single_page_allocation_smoke_test() {
        test_single_page_allocation();
    }

    #[test]
    /// smoke test single_page_stack
    fn single_page_stack_smoke_test() {
        test_single_page_stack();
    }

    #[test]
    /// tests construction and encoding in a new single page allocation
    fn single_page_allocation_from_encoded_allocation() {
        let i = 0b1010101010101010_0101010101010101;
        let single_page_allocation = SinglePageAllocation::from_encoded_allocation(i).unwrap();

        assert_eq!(0b1010101010101010, single_page_allocation.offset);
        assert_eq!(0b0101010101010101, single_page_allocation.length);
    }

    #[test]
    fn single_page_stack_from_encoded_test() {
        let i = 0b1010101010101010_0101010101010101;
        let single_page_stack = SinglePageStack::from_encoded_allocation(i);
        // stack top is offset + length
        assert_eq!(0b1111111111111111, single_page_stack.unwrap().top());
        let single_page_stack = SinglePageStack::from_encoded_allocation(0);
        // stack top is 0
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            single_page_stack.err().unwrap()
        );
        let i = 0b0000000000000001_0000000000000000;
        let single_page_stack = SinglePageStack::from_encoded_allocation(i);
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            single_page_stack.err().unwrap()
        );
    }

    #[test]
    /// tests that we can encode error return codes (zero length allocation)
    fn can_decode_encoded_allocation() {
        assert_eq!(
            // offset 0 = Success
            decode_encoded_allocation(0b0000000000000000_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Success,
        );
        assert_eq!(
            // offset 1 = generic error
            decode_encoded_allocation(0b0000000000000001_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Failure(RibosomeErrorCode::Unspecified),
        );
        assert_eq!(
            // offset 2 = serde json error
            decode_encoded_allocation(0b0000000000000010_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Failure(RibosomeErrorCode::ArgumentDeserializationFailed),
        );
        assert_eq!(
            // offset 3 = page overflow error
            decode_encoded_allocation(0b0000000000000011_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Failure(RibosomeErrorCode::OutOfMemory),
        );
        assert_eq!(
            // offset 4 = page overflow error
            decode_encoded_allocation(0b0000000000000100_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Failure(RibosomeErrorCode::ReceivedWrongActionResult),
        );

        assert_eq!(
            // nonsense offset = generic error
            decode_encoded_allocation(0b1010101010101010_0000000000000000).unwrap_err(),
            RibosomeReturnCode::Failure(RibosomeErrorCode::Unspecified),
        );
    }

    #[test]
    /// tests that a SinglePageAllocation returns its encoded offset/length pair as u32
    fn can_single_page_allocation_encode() {
        let i = 0b1010101010101010_0101010101010101;
        let spa = SinglePageAllocation::from_encoded_allocation(i).unwrap();

        assert_eq!(i, spa.encode());
    }

    #[test]
    fn can_single_page_allocation_new_fail() {
        assert_eq!(
            RibosomeErrorCode::ZeroSizedAllocation,
            SinglePageAllocation::new(0, 0).err().unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::new(1, 0).err().unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            SinglePageAllocation::new(<u16>::max_value(), <u16>::max_value())
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            SinglePageAllocation::new(<u16>::max_value(), 1)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::new(<u16>::max_value(), 0)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            SinglePageAllocation::new(1, <u16>::max_value())
                .err()
                .unwrap()
        );
        assert!(SinglePageAllocation::new(0, <u16>::max_value()).is_ok());
    }

    #[test]
    /// tests that a SinglePageAllocation returns its encoded offset/length pair as u32
    fn can_single_page_allocation_from_fail() {
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::from_encoded_allocation(0b0000000000000000_0000000000000000)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::from_encoded_allocation(0b0000000000000100_0000000000000000)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::from_encoded_allocation(<u32>::max_value())
                .err()
                .unwrap()
        );
    }

    #[test]
    fn test_u32_max_bits() {
        assert_eq!(<u16>::max_value(), super::u32_high_bits(<u32>::max_value()),);
        assert_eq!(<u16>::max_value(), super::u32_low_bits(<u32>::max_value()),);
        let upper_16: u32 = u32::from(<u16>::max_value());
        let upper_16 = upper_16 << 16;
        assert_eq!(<u16>::max_value(), super::u32_high_bits(upper_16),);
        assert_eq!(0, super::u32_low_bits(upper_16),);
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
