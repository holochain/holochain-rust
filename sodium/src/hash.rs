//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;

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
        let mut input = SecBuf::with_insecure(2);
        let mut output = SecBuf::with_insecure(32);
        {
            let mut input = input.write_lock();
            input[0] = 42;
            input[1] = 222;
        }
        {
            let mut input = input.write_lock();
            sha256(&mut input,&mut output);
        }
        let mut output = output.read_lock();
        assert_eq!("[193, 152, 204, 150, 33, 27, 103, 169, 2, 6, 174, 153, 35, 55, 117, 177, 84, 115, 121, 1, 166, 185, 242, 227, 116, 245, 129, 11, 9, 35, 188, 36]", format!("{:?}", *output));
    }

    #[test]
    fn it_should_sha512() {
        let mut input = SecBuf::with_insecure(2);
        let mut output = SecBuf::with_insecure(64);
        {
            let mut input = input.write_lock();
            input[0] = 42;
            input[1] = 222;
        }
        {
            let mut input = input.write_lock();
            sha512(&mut input,&mut output);
        }
        let mut output = output.write_lock();
        assert_eq!("[193, 152, 204, 150, 33, 27, 103, 169, 2, 6, 174, 153, 35, 55, 117, 177, 84, 115, 121, 1, 166, 185, 242, 227, 116, 245, 129, 11, 9, 35, 188, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", format!("{:?}", *output));
    }
}
