//! This module provides access to libsodium randomization functions

use super::check_init;

use super::secbuf::SecBuf;

/// randomize the provided SecBuf
pub fn buf(b: &mut SecBuf) {
    check_init();
    unsafe {
        let mut b = b.write_lock();
        rust_sodium_sys::randombytes_buf(rptr!(b), b.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_randomize_buffer() {
        let mut b = SecBuf::with_insecure(1);

        buf(&mut b);
    }
}
