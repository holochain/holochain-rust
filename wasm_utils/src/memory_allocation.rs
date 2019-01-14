use holochain_core_types::error::{RibosomeErrorCode, RibosomeReturnCode};
use holochain_core_types::bits_n_pieces::U16_MAX;
use holochain_core_types::bits_n_pieces::u32_merge_bits;
use holochain_core_types::error::RibosomeEncodedAllocation;
use holochain_core_types::bits_n_pieces::u32_split_bits;

//--------------------------------------------------------------------------------------------------
// Single Page Memory Allocation
//--------------------------------------------------------------------------------------------------



//--------------------------------------------------------------------------------------------------
// Single Page Memory Stack Manager
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {

    use super::*;
    use holochain_core_types::error::RibosomeReturnCode;

    pub fn test_single_page_allocation() -> SinglePageAllocation {
        SinglePageAllocation::new(0, 20).expect("could not create test SinglePageAllocation")
    }

    pub fn test_wasm_stack() -> WasmStack {
        WasmStack::new(test_single_page_allocation())
    }

    #[test]
    /// smoke test single_page_allocation
    fn single_page_allocation_smoke_test() {
        test_single_page_allocation();
    }

    #[test]
    /// smoke test wasm_stack
    fn wasm_stack_smoke_test() {
        test_wasm_stack();
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
    fn wasm_stack_from_encoded_test() {
        let i = 0b1010101010101010_0101010101010101;
        let wasm_stack = WasmStack::from_encoded_allocation(i);
        // stack top is offset + length
        assert_eq!(0b1111111111111111, wasm_stack.unwrap().top());
        let wasm_stack = WasmStack::from_encoded_allocation(0);
        // stack top is 0
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            wasm_stack.err().unwrap()
        );
        let i = 0b0000000000000001_0000000000000000;
        let wasm_stack = WasmStack::from_encoded_allocation(i);
        assert_eq!(
            RibosomeErrorCode::NotAnAllocation,
            wasm_stack.err().unwrap()
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

    #[test]
    fn test_u32_max_bits() {
        assert_eq!(u16::max_value(), super::u32_high_bits(<u32>::max_value()),);
        assert_eq!(u16::max_value(), super::u32_low_bits(<u32>::max_value()),);
        let upper_16: u32 = u32::from(u16::max_value());
        let upper_16 = upper_16 << 16;
        assert_eq!(u16::max_value(), super::u32_high_bits(upper_16),);
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
