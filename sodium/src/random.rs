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
        // Create two random buffers
        let mut buf_a = SecBuf::with_insecure(4);
        buf_a.randomize();
        let mut buf_b = SecBuf::with_insecure(4);
        buf_b.randomize();
        // Should be different (unless we are really lucky...)
        assert!(!buf_a.is_same(&mut buf_b));
        // re-randomize
        let mut buf_c = SecBuf::with_insecure(4);
        {
            let a = buf_a.read_lock();
            buf_c.from_array(&a).unwrap();
        }
        assert!(buf_a.is_same(&mut buf_c));
        buf_a.randomize();
        assert!(!buf_a.is_same(&mut buf_c));
    }
}
