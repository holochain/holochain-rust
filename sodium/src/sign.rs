//! This module provides access to libsodium
use super::check_init;

use super::secbuf::{
    SecBuf,
    Bufferable,
};

struct SeedKeypair {
    public_key:SecBuf,
    secret_key:SecBuf,
}
/// Generate a signing keypair from a seed buffer

/// @example
/// const { publicKey, secretKey } = mosodium.sign.seedKeypair(seed)
/// @param {SecBuf} seed - the seed to derive a keypair from
/// @retun {object} - { publicKey, privateKey }
pub fn seedKeypair(seed: &mut SecBuf) -> Vec<SecBuf> {
    check_init();
    unsafe {
        let mut public_key = SecBuf::with_insecure(rust_sodium_sys::crypto_sign_publickeybytes());
        let mut secret_key = SecBuf::with_secure(rust_sodium_sys::crypto_sign_secretkeybytes());
        {
        seed.writable();
        let _public_key = &mut public_key;
        let _secret_key = &mut secret_key;
        _public_key.writable();
        _secret_key.writable();
        let mut _public_key = _public_key.write_lock();
        let mut _secret_key = _secret_key.write_lock();
        rust_sodium_sys::crypto_sign_seed_keypair(raw_ptr_char!(_public_key),raw_ptr_char!(_secret_key),raw_ptr_char!(seed));
        }
        vec![public_key,secret_key]
    }
}

/// generate a signature
/// @example
/// const sig = mosodium.sign.sign(Buffer.from('hello'), secretKey)
/// @param {Buffer} message - the message to sign
/// @param {SecBuf} secretKey - the secret key to sign with
/// @return {Buffer} signature data
pub fn sign(message: &mut SecBuf,secret_key:&mut SecBuf){
    check_init();
    unsafe {
        let mut out = SecBuf::with_insecure(rust_sodium_sys::crypto_sign_bytes());
        secret_key.readable();
        let mut message = message.write_lock();
        let mut secret_key = secret_key.write_lock();
        let mut out = out.write_lock();
        let mut out_len = out.len();
        let mess_len = message.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_sign_detached(raw_ptr_char!(out),raw_ptr_longlong!(out_len),raw_ptr_char!(message),mess_len,raw_ptr_char!(secret_key));
        // return out;
    }
}



/// verify a signature given the message and a publicKey
/// @example
/// const isGood = mosodium.sign.verify(sig, Buffer.from('hello'), pubKey)
/// @param {Buffer} signature
/// @param {Buffer} message
/// @param {Buffer} publicKey
pub fn verify(signature: &mut SecBuf, message: &mut SecBuf, publicKey: &mut SecBuf)->i32{
    unsafe{
        let mut message = message.write_lock();
        let mut publicKey = publicKey.write_lock();
        let mess_len = message.len() as libc::c_ulonglong;
        return rust_sodium_sys::crypto_sign_verify_detached(raw_ptr_char!(signature), raw_ptr_char!(message),mess_len, raw_ptr_char!(publicKey))
    }
 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_sign_the_secBuf() {
        let mut b = SecBuf::with_insecure(1);

        {
            let mut b = b.write_lock();
            b[0] = 42;
        }

        let mut kp = seedKeypair(&mut b);
        // println!("{:?}", kp);
        {
            let b = b.read_lock();
            // let pk = kp[0].read_lock();
            let sk = &kp[1];
            assert_eq!(b[0], 42);
        }
        let signed = sign(&mut b, &mut kp[1]);
        {

        }
    }
}
