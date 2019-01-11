pub const U16_MAX: u32 = u16::max_value() as u32;
pub const U32_MAX: u64 = u32::max_value() as u64;

/// u16 high bits from u32
pub fn u32_high_bits(i: u32) -> u16 {
    (i >> 16) as u16
}

/// u32 high bits from u64
pub fn u64_high_bits(i: u64) -> u32 {
    (i >> 32) as u32
}

/// u16 low bits from u32
pub fn u32_low_bits(i: u32) -> u16 {
    // lossy cast
    (i as u16)
}

pub fn u64_low_bits(i: u64) -> u32 {
    // lossy cast
    (i as u32)
}

/// u32 as u16 (high, low) tuple
pub fn u32_split_bits(i: u32) -> (u16, u16) {
    (u32_high_bits(i), u32_low_bits(i))
}

/// u64 as u32 (high, low) tuple
pub fn u64_split_bits(i: u64) -> (u32, u32) {
    (u64_high_bits(i), u64_low_bits(i))
}

/// u16 (high, low) tuple into u32
pub fn u32_merge_bits(high: u16, low: u16) -> u32 {
    (u32::from(high) << 16) | u32::from(low)
}

/// u32 (high, low) tuple into u64
pub fn u64_merge_bits(high: u32, low: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}

#[cfg(test)]
pub mod bits_n_pieces_tests {

    #[test]
    fn u32_max_bits_test() {
        assert_eq!(u16::max_value(), super::u32_high_bits(<u32>::max_value()),);
        assert_eq!(u16::max_value(), super::u32_low_bits(<u32>::max_value()),);
        let upper_16: u32 = u32::from(u16::max_value());
        let upper_16 = upper_16 << 16;
        assert_eq!(u16::max_value(), super::u32_high_bits(upper_16),);
        assert_eq!(0, super::u32_low_bits(upper_16),);
    }

    #[test]
    fn u64_max_bits_test() {
        assert_eq!(u32::max_value(), super::u64_high_bits(<u64>::max_value()),);
        assert_eq!(u32::max_value(), super::u64_low_bits(<u64>::max_value()),);
        let upper_32: u64 = u64::from(u32::max_value());
        let upper_32 = upper_32 << 32;
        assert_eq!(u32::max_value(), super::u64_high_bits(upper_32),);
        assert_eq!(0, super::u64_low_bits(upper_32),);
    }

    #[test]
    /// tests that we can extract the high bits from a u32 into the correct u16
    fn u32_high_bits_test() {
        assert_eq!(
            0b1010101010101010,
            super::u32_high_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    fn u64_high_bits_test() {
        assert_eq!(
            0b1100_1100_1100_1100_1100_1100_1100_1100,
            super::u64_high_bits(0b11001100110011001100110011001100_00110011001100110011001100110011),
        );
    }

    #[test]
    /// tests that we can extract the high bits from a u32 into the correct u16
    fn u32_low_bits_test() {
        assert_eq!(
            0b0101010101010101,
            super::u32_low_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    fn u64_low_bits_test() {
        assert_eq!(
            0b0011_0011_0011_0011_0011_0011_0011_0011,
            super::u64_low_bits(0b11001100110011001100110011001100_00110011001100110011001100110011),
        );
    }

    #[test]
    fn u32_split_bits_test() {
        assert_eq!(
            (0b1010101010101010, 0b0101010101010101),
            super::u32_split_bits(0b1010101010101010_0101010101010101),
        );
    }

    #[test]
    fn u64_split_bits_test() {
        assert_eq!(
            (0b1100_1100_1100_1100_1100_1100_1100_1100, 0b0011_0011_0011_0011_0011_0011_0011_0011),
            super::u64_split_bits(0b11001100110011001100110011001100_00110011001100110011001100110011),
        );
    }

    #[test]
    fn u32_merge_bits_test() {
        assert_eq!(
            0b1010101010101010_0101010101010101,
            super::u32_merge_bits(0b1010101010101010, 0b0101010101010101),
        );
    }

    #[test]
    fn u64_merge_bits_test() {
        assert_eq!(
            0b11001100110011001100110011001100_00110011001100110011001100110011,
            super::u64_merge_bits(0b1100_1100_1100_1100_1100_1100_1100_1100, 0b0011_0011_0011_0011_0011_0011_0011_0011),
        );
    }

}
