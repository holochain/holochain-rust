//! This module provides access to libsodium

use super::{check_init, secbuf::SecBuf};
use crate::error::SodiumError;

/// Size of return value while converting to sha256
pub const BYTES256: usize = rust_sodium_sys::crypto_hash_sha256_BYTES as usize;

/// Size of return value while converting to sha512
pub const BYTES512: usize = rust_sodium_sys::crypto_hash_sha512_BYTES as usize;

/// Compute the sha256 hash of input buffer
/// ****
/// @param {SecBuf} input - the data to hash
///
/// @param {SecBuf} output - Empty Buffer to be used as output
pub fn sha256(input: &mut SecBuf, output: &mut SecBuf) -> Result<(), SodiumError>  {
    check_init();
    let input_len = input.len() as libc::c_ulonglong;
    let input = input.read_lock();
    let mut output = output.write_lock();
    unsafe {
        rust_sodium_sys::crypto_hash_sha256(
            raw_ptr_char!(output),
            raw_ptr_char_immut!(input),
            input_len,
        );
    }
    Ok(())
}

/// Compute the sha512 hash of input buffer
/// ****
/// @param {Buffer} input - the data to hash
///
/// @param {SecBuf} output - Empty Buffer to be used as output
pub fn sha512(input: &mut SecBuf, output: &mut SecBuf) -> Result<(), SodiumError>  {
    check_init();
    let input = input.read_lock();
    let mut output = output.write_lock();
    let input_len = input.len() as libc::c_ulonglong;
    unsafe {
        rust_sodium_sys::crypto_hash_sha512(
            raw_ptr_char!(output),
            raw_ptr_char_immut!(input),
            input_len,
        );
    }
    Ok(())
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
            sha256(&mut input, &mut output).unwrap();
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
            sha512(&mut input, &mut output).unwrap();
        }
        let output = output.write_lock();
        assert_eq!("[7, 117, 152, 125, 243, 201, 32, 78, 241, 175, 174, 114, 145, 29, 183, 142, 198, 91, 47, 209, 111, 35, 223, 28, 65, 246, 126, 147, 48, 171, 241, 88, 26, 108, 130, 55, 221, 6, 221, 45, 125, 138, 41, 184, 144, 190, 203, 31, 96, 247, 207, 176, 74, 129, 12, 29, 134, 172, 216, 180, 31, 1, 61, 59]", format!("{:?}", *output));
    }
}
