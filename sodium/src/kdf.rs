//! This module provides access to libsodium

use super::{check_init, secbuf::SecBuf};
use crate::{
    error::{SodiumError, SodiumResult},
    util::check_buf_len,
};

pub const CONTEXTBYTES: usize = rust_sodium_sys::crypto_kdf_CONTEXTBYTES as usize;

/// Derive a subkey from a parent key
/// ****
/// @param {SecBuf} out - Empty Buffer to be used as output
///
/// @param {number} index - subkey index
///
/// @param {Buffer} context - eight bytes context
///
/// @param {SecBuf} parent - the parent key to derive from

pub fn derive(
    out: &mut SecBuf,
    index: u64,
    context: &mut SecBuf,
    parent: &mut SecBuf,
) -> SodiumResult<()> {
    check_init();
    {
        let out = out.read_lock();
        let o = out.len();
        let context = context.read_lock();
        let c = context.len();
        if check_buf_len(o) {
            return Err(SodiumError::OutputLength(format!(
                "Invalid 'out' Buffer length:{}",
                o
            )));
        } else if c != CONTEXTBYTES {
            return Err(SodiumError::OutputLength(format!(
                "context must be a Buffer of length: {}.",
                CONTEXTBYTES
            )));
        }
    }
    unsafe {
        let mut out = out.write_lock();
        let parent = parent.read_lock();
        let context = context.read_lock();
        rust_sodium_sys::crypto_kdf_derive_from_key(
            raw_ptr_char!(out),
            out.len(),
            index,
            raw_ptr_ichar_immut!(context),
            raw_ptr_char_immut!(parent),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::buf;

    #[test]
    fn it_should_derive_consistantly() {
        let mut context = SecBuf::with_secure(CONTEXTBYTES);
        let mut parent = SecBuf::with_secure(32);
        buf(&mut context);
        buf(&mut parent);
        let mut out1 = SecBuf::with_secure(32);
        let mut out2 = SecBuf::with_secure(32);
        {
            let mut out1 = out1.write_lock();
            derive(&mut out1, 3, &mut context, &mut parent).unwrap();
        }
        {
            let mut out2 = out2.write_lock();
            derive(&mut out2, 3, &mut context, &mut parent).unwrap();
        }
        let out1 = out1.read_lock();
        let out2 = out2.read_lock();
        assert_eq!(format!("{:?}", *out1), format!("{:?}", *out2));
    }
    #[test]
    fn it_should_return_error_on_bad_output_buffer() {
        let mut context = SecBuf::with_secure(8);
        let mut parent = SecBuf::with_secure(32);
        buf(&mut context);
        buf(&mut parent);
        let mut out = SecBuf::with_insecure(2);
        {
            let mut out = out.write_lock();
            match derive(&mut out, 3, &mut context, &mut parent) {
                Ok(_k) => {
                    assert!(false);
                }
                Err(_e) => {
                    assert!(true);
                }
            };
        }
    }
}
