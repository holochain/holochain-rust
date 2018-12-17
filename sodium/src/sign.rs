//! This module provides access to libsodium
use super::check_init;

use super::secbuf::{
    SecBuf,
};

use super::random::{
    buf,
};

pub fn seedKeypair(public_key: &mut SecBuf,secret_key: &mut SecBuf,seed: &mut SecBuf) {
    check_init();
    println!("<Seed_Generation/>");
    unsafe {
        // public_key.writable();
        // secret_key.writable();
        // seed.writable();
        let mut seed = seed.write_lock();
        let mut secret_key = secret_key.write_lock();
        let mut public_key = public_key.write_lock();
        rust_sodium_sys::crypto_sign_seed_keypair(raw_ptr_char!(public_key),raw_ptr_char!(secret_key),raw_ptr_char!(seed));
        }
        // println!("{:?}",secret_key );
     // (public_key.into(),secret_key.into())
}

pub fn sign(message: &mut SecBuf,secret_key:&mut SecBuf)->SecBuf{
    check_init();
    println!("<Signing/>");
    let mut out = SecBuf::with_secure(rust_sodium_sys::crypto_sign_BYTES as usize);
    unsafe {
        // secret_key.readable();
        let mut message = message.write_lock();
        let mut secret_key = secret_key.write_lock();
        let mut out = out.write_lock();
        let mut out_len = out.len();
        let mess_len = message.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_sign_detached(raw_ptr_char!(out),raw_ptr_longlong!(out_len),raw_ptr_char!(message),mess_len,raw_ptr_char!(secret_key));
    }
    return out;
}



pub fn verify(signature: &mut SecBuf, message: &mut SecBuf, public_key: &mut SecBuf)->i32{
    println!("<Verify/>");
    unsafe{
        let mut message = message.write_lock();
        let mut public_key = public_key.write_lock();
        let mess_len = message.len() as libc::c_ulonglong;
        return rust_sodium_sys::crypto_sign_verify_detached(raw_ptr_char!(signature), raw_ptr_char!(message),mess_len, raw_ptr_char!(public_key))
    }
 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_get_true_on_good_verify() {
        let mut seed = SecBuf::with_secure(32);
        let mut public_key = SecBuf::with_secure(32);
        let mut secret_key = SecBuf::with_secure(64);
        {
            let mut seed = seed.write_lock();
            let mut secret_key = secret_key.write_lock();
            let mut public_key = public_key.write_lock();
            buf(&mut seed);
            buf(&mut public_key);
            buf(&mut secret_key);
        }
        {
            let mut seed = seed.write_lock();
            let mut secret_key = secret_key.write_lock();
            let mut public_key = public_key.write_lock();
            seedKeypair(&mut public_key,&mut secret_key,&mut seed);
        }

        {
            let mut secret_key = secret_key.write_lock();
            assert_eq!(34,secret_key.len());
        }
            let mut message = SecBuf::with_insecure(32);
        {
            let mut message = message.write_lock();
            buf(&mut message);
        }
        {
            let mut message = message.write_lock();
            let mut secret_key = secret_key.write_lock();
            // let mut sig = sign(&mut message,&mut secret_key);
        }// let ver = verify(&mut sig,&mut message,&mut public_key);

        // assert_eq!(1, ver);
    }

    // #[test]
    // fn it_should_get_false_on_bad_verify() {
    // }

}
