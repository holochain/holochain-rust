//! This module provides access to libsodium utility and memory functions

use super::check_init;

use super::secbuf::SecBuf;

/// zero all memory within the provided SecBuf
pub fn zero(b: &mut SecBuf) {
    check_init();
    unsafe {
        let mut b = b.write_lock();
        rust_sodium_sys::sodium_memzero(rptr!(b), b.len());
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

        zero(&mut b);

        {
            let b = b.read_lock();
            assert_eq!(0, b[0]);
        }
    }
}
