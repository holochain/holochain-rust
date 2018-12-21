//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;

pub const CONTEXTBYTES: usize = rust_sodium_sys::crypto_kdf_CONTEXTBYTES as usize;


/// Derive a subkey from a parent key
/// ****
/// @example
/// const subkey = mosodium.kdf.derive(1, Buffer.from('eightchr'), pk)
/// ****
/// @param {number} index - subkey index
/// @param {Buffer} context - eight bytes context
/// @param {SecBuf} parent - the parent key to derive from
/// @return {SecBuf}

pub fn derive(out: &mut SecBuf, index: u64, context: &mut SecBuf, parent: &mut SecBuf) {
    unsafe {
        let mut out = out.write_lock();
        let mut parent = parent.read_lock();
        let mut context = context.read_lock();
        if context.len() !=  CONTEXTBYTES {
            panic!("context must be a Buffer of length: {}.",CONTEXTBYTES);
        }
        rust_sodium_sys::crypto_kdf_derive_from_key(raw_ptr_char!(out),out.len(),index,raw_ptr_ichar_immut!(context),raw_ptr_char_immut!(parent));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_derive_consistantly() {
        let mut context = SecBuf::with_secure(8);
        let mut parent = SecBuf::with_secure(32);
        buf(&mut context);
        buf(&mut parent);
        let mut out1 = SecBuf::with_secure(32);
        let mut out2 = SecBuf::with_secure(32);
        {
            let mut out1 = out1.write_lock();
            derive(&mut out1,3,&mut context,&mut parent);
        }
        {
            let mut out2 = out2.write_lock();
            derive(&mut out2,3,&mut context,&mut parent);
        }
        let mut out1 = out1.read_lock();
        let mut out2 = out2.read_lock();
        assert_eq!(format!("{:?}", *out1), format!("{:?}", *out2));
    }

}
