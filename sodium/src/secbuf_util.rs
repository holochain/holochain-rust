//! This module provides access to libsodium utility and memory functions

use super::{check_init, secbuf::SecBuf};
use crate::error::SodiumError;

/// Check if length of buffer is of approprate size
/// it should be either of size 8,16,32 or 64
pub fn check_buf_len(sb: usize) -> bool {
    sb != 8 && sb != 16 && sb != 32 && sb != 64
}

impl SecBuf {
    /// Return true if memory is only zeroes, i.e. [0,0,0,0,0,0,0,0]
    fn is_zero(&mut self) -> bool {
        check_init();
        let mut a = self.write_lock();
        unsafe { rust_sodium_sys::sodium_is_zero(raw_ptr_char!(a), a.len()) == 1 }
    }

    /// Zero all memory
    pub fn zero(&mut self) {
        check_init();
        let mut b = self.write_lock();
        unsafe {
            rust_sodium_sys::sodium_memzero(raw_ptr_void!(b), b.len());
        }
    }

    /// Increments all memory by 1
    pub fn increment(&mut self) {
        check_init();
        let mut b = self.write_lock();
        unsafe {
            rust_sodium_sys::sodium_increment(raw_ptr_char!(b), b.len());
        }
    }

    /// Compares the Two SecBuf
    /// Return :
    /// | if a > b; return 1
    /// | if a < b; return -1
    /// | if a == b; return 0
    pub fn compare(&mut self, b: &mut SecBuf) -> i32 {
        check_init();
        let mut a = self.write_lock();
        let mut b = b.write_lock();
        unsafe { rust_sodium_sys::sodium_compare(raw_ptr_char!(a), raw_ptr_char!(b), a.len()) }
    }

    /// Load the [u8] into the SecBuf
    pub fn from_array(&mut self, data: &[u8]) -> Result<(), SodiumError> {
        if (data.len() != self.len()) {
            return Err(SodiumError::Generic(
                "Input does not have same size as SecBuf".to_string(),
            ));
        }
        self.write(0, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_zero_buffer() {
        let mut b = SecBuf::with_insecure(1);
        {
            let mut b = b.write_lock();
            b[0] = 42;
        }
        b.zero();
        {
            let b = b.read_lock();
            assert_eq!(0, b[0]);
        }
    }

    #[test]
    fn it_should_increment_buffer() {
        let mut b = SecBuf::with_insecure(1);
        {
            let mut b = b.write_lock();
            b[0] = 42;
        }
        b.increment();
        {
            let b = b.read_lock();
            assert_eq!(43, b[0]);
        }
    }

    #[test]
    fn it_should_compare_buffer() {
        let mut a = SecBuf::with_insecure(1);
        {
            let mut a = a.write_lock();
            a[0] = 50;
        }
        let mut b = SecBuf::with_insecure(1);
        {
            let mut b = b.write_lock();
            b[0] = 45;
        }
        let mut c = SecBuf::with_insecure(1);
        {
            let mut c = c.write_lock();
            c[0] = 45;
        }
        let val_1 = a.compare(&mut b);
        let val_2 = b.compare(&mut a);
        let val_3 = b.compare(&mut c);
        assert_eq!(1, val_1);
        assert_eq!(-1, val_2);
        assert_eq!(0, val_3);
    }

    #[test]
    fn it_should_be_zero() {
        let mut buf = SecBuf::with_insecure(4);
        assert!(buf.is_zero());
        buf.increment();
        assert!(!buf.is_zero());
        buf.zero();
        assert!(buf.is_zero());
    }

    #[test]
    fn it_should_from_array() {
        let mut b = SecBuf::with_insecure(4);
        let bad_array = vec![42];
        let good_array = vec![0, 1, 2, 3];
        // Wrong size should give error
        let res = b.from_array(&bad_array);
        assert!(res.is_err());
        // Correct size should copy
        b.from_array(&good_array).unwrap();
        let b = b.read_lock();
        assert_eq!("[0, 1, 2, 3]", format!("{:?}", *b));
    }
}
