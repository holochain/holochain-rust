//! This module provides access to libsodium

use super::secbuf::SecBuf;
use crate::error::SodiumResult;

/// Size of return value while converting to sha256
pub const BYTES256 : usize = rust_sodium_sys::crypto_hash_sha256_BYTES as usize;

/// Size of return value while converting to sha512
pub const BYTES512 : usize = rust_sodium_sys::crypto_hash_sha512_BYTES as usize;

/// Compute the sha256 hash of input buffer
/// ****
/// @param {SecBuf} input - the data to hash
///
/// @param {SecBuf} output - Empty Buffer to be used as output
pub fn sha256(input: &mut SecBuf,output: &mut SecBuf)->SodiumResult<()> {
    unsafe {
        let input_len = input.len() as libc::c_ulonglong;
        let input = input.read_lock();
        let mut output = output.write_lock();
        rust_sodium_sys::crypto_hash_sha256(raw_ptr_char!(output),raw_ptr_char_immut!(input),input_len);
        Ok(())
    }
}

/// Compute the sha512 hash of input buffer
/// ****
/// @param {Buffer} input - the data to hash
///
/// @param {SecBuf} output - Empty Buffer to be used as output
pub fn sha512(input: &mut SecBuf,output: &mut SecBuf)->SodiumResult<()> {
    unsafe {
        let input = input.read_lock();
        let mut output = output.write_lock();
        let input_len = input.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_hash_sha256(raw_ptr_char!(output),raw_ptr_char_immut!(input),input_len);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_sha256() {
        let mut input = SecBuf::with_insecure(2);
        let mut output = SecBuf::with_insecure(BYTES256);
        {
            let mut input = input.write_lock();
            input[0] = 42;
            input[1] = 222;
        }
        {
            let mut input = input.write_lock();
            sha256(&mut input,&mut output).unwrap();
        }
        let output = output.read_lock();
        assert_eq!("[193, 152, 204, 150, 33, 27, 103, 169, 2, 6, 174, 153, 35, 55, 117, 177, 84, 115, 121, 1, 166, 185, 242, 227, 116, 245, 129, 11, 9, 35, 188, 36]", format!("{:?}", *output));
    }

    #[test]
    fn it_should_sha512() {
        let mut input = SecBuf::with_insecure(2);
        let mut output = SecBuf::with_insecure(BYTES512);
        {
            let mut input = input.write_lock();
            input[0] = 42;
            input[1] = 222;
        }
        {
            let mut input = input.write_lock();
            sha512(&mut input,&mut output).unwrap();
        }
        let output = output.write_lock();
        assert_eq!("[193, 152, 204, 150, 33, 27, 103, 169, 2, 6, 174, 153, 35, 55, 117, 177, 84, 115, 121, 1, 166, 185, 242, 227, 116, 245, 129, 11, 9, 35, 188, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", format!("{:?}", *output));
    }
}
