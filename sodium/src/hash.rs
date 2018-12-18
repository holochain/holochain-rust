//! This module provides access to libsodium
use super::check_init;

use super::secbuf::{
    SecBuf,
};

use super::random::{
    buf,
};


/// XOR an arbitrary length buffer (byteLength must be a multiple of 4)
/// into an int32 sized javascript number
/// ****
/// @example
/// const myInt = mosodium.hash.toInt(mosodium.hash.sha256(Buffer.from('hello')))
/// ****
/// @param {Buffer} input - the data to xor
/// @return {number}
// TODO : readInt32LE ()

// pub fn toInt(input: &mut SecBuf,output: &mut SecBuf) {
//     unsafe {
//         let mut input = input.read_lock();
//         let mut output = output.write_lock();
//         let input_len = input.len() as libc::c_ulonglong;
//     }
// }

/// Compute the sha256 hash of input buffer
/// ****
/// @example
/// const hash = mosodium.hash.sha256(Buffer.from('hello'))
/// ****
/// @param {Buffer} input - the data to hash
/// @return {Buffer}
pub fn sha256(input: &mut SecBuf,output: &mut SecBuf) {
    unsafe {
        let input_len = input.len() as libc::c_ulonglong;
        let mut input = input.write_lock();
        let mut output = output.write_lock();
        rust_sodium_sys::crypto_hash_sha256(raw_ptr_char!(output),raw_ptr_char_immut!(input),input_len);
    }
}

/// Compute the sha512 hash of input buffer
/// ****
/// @example
/// const hash = mosodium.hash.sha512(Buffer.from('hello'))
/// ****
/// @param {Buffer} input - the data to hash
/// @return {Buffer}

pub fn sha512(input: &mut SecBuf,output: &mut SecBuf) {
    unsafe {
        let mut input = input.read_lock();
        let mut output = output.write_lock();
        let input_len = input.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_hash_sha256(raw_ptr_char!(output),raw_ptr_char_immut!(input),input_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_sha256() {
        let mut input = SecBuf::with_secure(32);
        let mut output = SecBuf::with_secure(32);
        buf(&mut input);
        {
            let mut input = input.write_lock();
            sha256(&mut input,&mut output);
        }
        let mut input = input.write_lock();
        let mut output = output.write_lock();
        println!("Input: {:?}",input );
        println!("Output: {:?}",output );
        assert_eq!(32, output.len());
    }
    #[test]
    fn it_should_sha512() {
        let mut input = SecBuf::with_secure(64);
        let mut output = SecBuf::with_secure(64);
        buf(&mut input);
        {
            let mut input = input.write_lock();
            sha512(&mut input,&mut output);
        }
        let mut input = input.write_lock();
        let mut output = output.write_lock();
        println!("Input: {:?}",input );
        println!("Output: {:?}",output );
        assert_eq!(64, output.len());
    }
}
