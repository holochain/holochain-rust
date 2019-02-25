//! This module provides access to libsodium randomization functions

use super::check_init;

use super::secbuf::SecBuf;

impl SecBuf {
    /// randomize the provided SecBuf
    pub fn randomize(&mut self) {
        check_init();
        unsafe {
            let mut b = self.write_lock();
            rust_sodium_sys::randombytes_buf(raw_ptr_void!(b), b.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_randomize_buffer() {
        let mut buf_a = SecBuf::with_insecure(4);
        buf_a.randomize();
        let mut buf_b = SecBuf::with_insecure(4);
        buf_b.randomize();
        assert_ne!(buf_a.dump(), buf_b.dump());
        // re-randomize
        let mut buf_c = SecBuf::with_insecure(4);
        buf_c.from_array(&buf_a.dump()).unwrap();
        assert_eq!(buf_c.dump(), buf_a.dump());
        buf_a.randomize();
        assert_ne!(buf_c.dump(), buf_a.dump());
    }
}
