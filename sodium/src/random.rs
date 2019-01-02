//! This module provides access to libsodium randomization functions

use super::check_init;

use super::secbuf::SecBuf;

/// randomize the provided SecBuf
pub fn random_secbuf(b: &mut SecBuf) {
    check_init();
    unsafe {
        let mut b = b.write_lock();
        rust_sodium_sys::randombytes_buf(raw_ptr_void!(b), b.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_randomize_buffer() {
        let mut b = SecBuf::with_insecure(1);
        random_secbuf(&mut b);
    }
}
