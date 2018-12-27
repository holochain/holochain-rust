//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;

pub const NONCEBYTES : usize = rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_NPUBBYTES as usize;
pub const ABYTES : usize = rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_ABYTES as usize;

/// Generate symmetric cipher text given a message, secret, and optional auth data
/// ****
/// @example
/// const cipher = mosodium.aead.enc(Buffer.from('hello'), secret)
/// ****
/// @param {Buffer} message - data to encrypt
/// @param {SecBuf} secret - symmetric secret key
/// @param {Buffer} adata - optional additional authenticated data
/// @return {object} - { nonce, cipher }


pub fn enc(message: &mut SecBuf,nonce: &mut SecBuf,cipher: &mut SecBuf,secret: &mut SecBuf,adata: Option<&mut SecBuf>){
    match adata {
        Some(adata) => {
            let mut message = message.write_lock();
            let mut nonce = nonce.write_lock();
            let mut cipher = cipher.write_lock();
            let mut secret = secret.write_lock();
            let mut adata = adata.write_lock();
            let ad_len = adata.len() as libc::c_ulonglong;
            encrypt(&mut message,&mut nonce,&mut cipher,&mut secret,&mut adata,ad_len)
        }
        None => {
            // let mut adata = adata.write_lock();
            // let ad_len = std::ptr::null_mut();
            // encrypt(&mut message,&mut nonce,&mut cipher,&mut secret,&mut adata,ad_len)
        }
    }
}


pub fn encrypt(message: &mut SecBuf,nonce: &mut SecBuf,cipher: &mut SecBuf,secret: &mut SecBuf,adata: &mut SecBuf, ad_len: libc::c_ulonglong) {
    unsafe {
        let mut secret = secret.read_lock();
        let mut message = message.read_lock();
        let _mess_len = message.len() as libc::c_ulonglong;
        let mut adata = adata.read_lock();
        let mut k = SecBuf::with_secure(32);
        let mut k = k.read_lock();
        let mut nonce = nonce.read_lock();
        let mut cipher = cipher.write_lock();
        let mut ci_len = cipher.len();
        let mut len : *mut libc::c_ulonglong;
        len = ci_len as *mut libc::c_ulonglong;
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_encrypt(raw_ptr_char!(cipher),len,raw_ptr_char_immut!(message),_mess_len,raw_ptr_char_immut!(adata),ad_len,raw_ptr_char_immut!(secret),raw_ptr_char_immut!(nonce),raw_ptr_char_immut!(k));
    }
}

/// Decrypt symmetric cipher text given a nonce, secret, and optional auth data
/// ****
/// @example
/// const decrypted_message = mosodium.aead.dec(nonce, cipher, secret)
/// ****
/// @param {Buffer} nonce - sometimes called initialization vector (iv)
/// @param {Buffer} cipher - the cipher text
/// @param {SecBuf} secret - symmetric secret key
/// @param {Buffer} adata - optional additional authenticated data
/// @return {Buffer} - decrypted_message
pub fn dec(decrypted_message: &mut SecBuf,nonce: &mut SecBuf, cipher: &mut SecBuf, secret: &mut SecBuf, adata: &mut SecBuf){
    unsafe {
        let mut secret = secret.write_lock();
        let mut cipher = cipher.read_lock();
        let cipher_len = cipher.len() as libc::c_ulonglong;
        let mut adata = adata.read_lock();
        let ad_len = adata.len() as libc::c_ulonglong;
        let mut k = SecBuf::with_secure(32);
        let mut k = k.read_lock();
        let mut nonce = nonce.read_lock();
        let mut decrypted_message = decrypted_message.write_lock();
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_decrypt(raw_ptr_char!(decrypted_message),std::ptr::null_mut(),raw_ptr_char!(secret),raw_ptr_char_immut!(cipher),cipher_len,raw_ptr_char_immut!(adata),ad_len,raw_ptr_char_immut!(nonce),raw_ptr_char_immut!(k));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_with_auth_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        let mut nonce = SecBuf::with_secure(32);
        let mut cipher = SecBuf::with_secure(32);
        let mut adata = SecBuf::with_secure(16);
        println!("----------------");
        buf(&mut message);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut nonce,&mut cipher,&mut secret,Some(&mut adata));
        }
        println!("----------------");
        let mut decrypted_message = SecBuf::with_secure(16);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut nonce,&mut cipher,&mut secret,&mut adata);
        }
        println!("----------------");
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!(format!("{:?}", *message), format!("{:?}", *decrypted_message));
        println!("----------------");
    }
    #[test]
    fn it_should_with_bad_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        let mut nonce = SecBuf::with_secure(32);
        let mut cipher = SecBuf::with_secure(32);
        let mut adata = SecBuf::with_secure(16);
        let mut adata1 = SecBuf::with_secure(16);
        buf(&mut adata1);
        buf(&mut message);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut nonce,&mut cipher,&mut secret,Some(&mut adata));
        }
        let mut decrypted_message = SecBuf::with_secure(16);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut nonce,&mut cipher,&mut secret,&mut adata1);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!("[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", format!("{:?}", *decrypted_message));
    }
}
