use holochain_core_types::error::{RibosomeErrorCode, RibosomeReturnCode};
use bits_n_pieces::u32_split_bits;
use bits_n_pieces::U16_MAX;
use bits_n_pieces::u32_merge_bits;

pub struct EncodedAllocation32(u32)

pub struct EncodedAllocation64(u64)

pub struct Allocation32 {
    offset: u16,
    length: u16,
}

impl Allocation32 {
    pub fn new(offset: u16, length: u16) -> Result<Self, RibosomeErrorCode> {
        if (u32::from(offset) + u32::from(length)) > U16_MAX {
            Err(RibosomeErrorCode::OutOfMemory)
        } else if (offset + length) == 0 {
            Err(RibosomeErrorCode::ZeroSizedAllocation)
        } else if length == 0 {
            Err(RibosomeErrorCode::NotAnAllocation)
        } else {
            Ok(Self { offset, length })
        }
    }
}

impl TryFrom<RibosomeReturnCode32> for Allocation32 {
    type Error = RibosomeReturnCode32;
    fn try_from(return_code: RibosomeReturnCode32) -> Result<Self, Self::Error> {
        let (offset, length) = split_u32(u32::from(return_code));
        match return_code {
            Allocation::(_) => {
                Ok(Self { offset, length })
            },
            _ => Err(RibosomeReturnCode32::from_offset(offset),
        }
    }
}

pub struct Allocation64 {

}


//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

pub fn decode_encoded_allocation(
    encoded_allocation: u32,
) -> Result<SinglePageAllocation, RibosomeReturnCode> {

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

impl SinglePageAllocation {


    /// An Encoded Allocation is a u32 where 'offset' is first 16-bits and 'length' last 16-bits
    /// A valid allocation must not have a length of zero
    /// An Encoded Allocation with an offset but no length is actually an encoding of an ErrorCode
    pub fn from_encoded_allocation(encoded_allocation: u32) -> Result<Self, RibosomeErrorCode> {
        decode_encoded_allocation(encoded_allocation)
            .map_err(|_| RibosomeErrorCode::NotAnAllocation)
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
        assert!(u32::from(self.top) + u32::from(size) <= U16_MAX);
        let offset = self.top;
        self.top += size;
        offset
    }

    pub fn deallocate(&mut self, allocation: SinglePageAllocation) -> Result<(), ()> {
        // TODO: This method should not return an empty error.
        if self.top == allocation.offset + allocation.length {
            self.top = allocation.offset;

            Ok(())
        } else {
            Err(())
        }
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
            SinglePageAllocation::new(u16::max_value(), u16::max_value())
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            SinglePageAllocation::new(u16::max_value(), 1)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            SinglePageAllocation::new(u16::max_value(), 0)
                .err()
                .unwrap()
        );
        assert_eq!(
            RibosomeErrorCode::OutOfMemory,
            SinglePageAllocation::new(1, u16::max_value())
                .err()
                .unwrap()
        );
        assert!(SinglePageAllocation::new(0, u16::max_value()).is_ok());
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

}
