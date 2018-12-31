//! This module provides access to libsodium

use super::secbuf::SecBuf;
use super::random::buf;
use crate::error::{
    SodiumResult,
    SodiumError,
};
use crate::util::check_buf_len;

pub const NONCEBYTES : usize = rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_NPUBBYTES as usize;
pub const ABYTES : usize = rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_ABYTES as usize;

/// Generate symmetric cipher text given a message, secret, and optional auth data
/// ****
/// @param {SecBuf} message - data to encrypt
/// @param {SecBuf} secret - symmetric secret key
/// @param {SecBuf} adata - optional additional authenticated data
/// @param {SecBuf} nonce - Empty Buffer to be used as output
/// @param {SecBuf} cipher - Empty Buffer to be used as output

pub fn enc(message: &mut SecBuf,secret: &mut SecBuf,adata: Option<&mut SecBuf>,nonce: &mut SecBuf,cipher: &mut SecBuf)->SodiumResult<()>{
    {
        // Checking output Buffers
        let mut nonce = nonce.write_lock();
        let mut cipher = cipher.write_lock();
        let n = nonce.len();
        let c = cipher.len();
        if check_buf_len(n){
            return Err(SodiumError::OutputLength(format!("Invalid nonce Buffer length:{}", n)));

        }else if check_buf_len(c){
            return Err(SodiumError::OutputLength(format!("Invalid cipher Buffer length:{}", c)));
        }
    }
    match adata {
        Some(adata) => {
            let mut message = message.write_lock();
            let mut nonce = nonce.write_lock();
            let mut cipher = cipher.write_lock();
            let mut secret = secret.write_lock();
            let mut adata = adata.write_lock();
            let ad_len = adata.len() as libc::c_ulonglong;
            encrypt(&mut message,&mut secret,&mut adata,ad_len,&mut nonce,&mut cipher);
            Ok(())
        }
        None => {
            let mut adata = SecBuf::with_insecure(1);
            let mut adata = adata.write_lock();
            let ad_len : *const u64 = std::ptr::null();
            let ad_len = ad_len as u64;
            let mut message = message.write_lock();
            let mut nonce = nonce.write_lock();
            let mut cipher = cipher.write_lock();
            let mut secret = secret.write_lock();
            encrypt(&mut message,&mut secret,&mut adata,ad_len,&mut nonce,&mut cipher);
            Ok(())
        }
    }
}


pub fn encrypt(message: &mut SecBuf,secret: &mut SecBuf,adata: &mut SecBuf, ad_len: libc::c_ulonglong,nonce: &mut SecBuf,cipher: &mut SecBuf)->SodiumResult<()>{
    unsafe {
        let mut secret = secret.read_lock();
        let mut message = message.read_lock();
        let _mess_len = message.len() as libc::c_ulonglong;
        let mut adata = adata.read_lock();
        let mut k = SecBuf::with_secure(32);
        let mut k = k.read_lock();
        let mut nonce = nonce.read_lock();
        let mut cipher = cipher.write_lock();
        let mut ci_len = cipher.len() as libc::c_ulonglong;
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_encrypt(raw_ptr_char!(cipher),&mut ci_len,raw_ptr_char_immut!(message),_mess_len,raw_ptr_char_immut!(adata),ad_len,raw_ptr_char_immut!(secret),raw_ptr_char_immut!(nonce),raw_ptr_char_immut!(k));
        Ok(())
    }
}

/// Decrypt symmetric cipher text given a nonce, secret, and optional auth data
/// ****
/// @param {SecBuf} decrypted_message - Empty Buffer to be used as output to return the result
/// @param {SecBuf} secret - symmetric secret key
/// @param {Buffer} adata - optional additional authenticated data
/// @param {Buffer} nonce - sometimes called initialization vector (iv)
/// @param {Buffer} cipher - the cipher text

pub fn dec(decrypted_message: &mut SecBuf,secret: &mut SecBuf,adata: Option<&mut SecBuf>,nonce: &mut SecBuf,cipher: &mut SecBuf)->SodiumResult<()>{
    match adata {
        Some(adata) => {
            let mut decrypted_message = decrypted_message.write_lock();
            let mut nonce = nonce.write_lock();
            let mut cipher = cipher.write_lock();
            let mut secret = secret.write_lock();
            let mut adata = adata.write_lock();
            let ad_len = adata.len() as libc::c_ulonglong;
            decrypt(&mut decrypted_message,&mut secret,&mut adata,ad_len,&mut nonce,&mut cipher);
            Ok(())
        }
        None => {
            let mut adata = SecBuf::with_insecure(1);
            let mut adata = adata.write_lock();
            let ad_len : *const u64 = std::ptr::null();
            let ad_len = ad_len as u64;
            let mut decrypted_message = decrypted_message.write_lock();
            let mut nonce = nonce.write_lock();
            let mut cipher = cipher.write_lock();
            let mut secret = secret.write_lock();
            decrypt(&mut decrypted_message,&mut secret,&mut adata,ad_len,&mut nonce,&mut cipher);
            Ok(())
        }
    }
}
pub fn decrypt(decrypted_message: &mut SecBuf, secret: &mut SecBuf, adata: &mut SecBuf, ad_len: libc::c_ulonglong,nonce: &mut SecBuf, cipher: &mut SecBuf)->SodiumResult<()>{
    unsafe {
        let mut secret = secret.write_lock();
        let mut cipher = cipher.read_lock();
        let cipher_len = cipher.len() as libc::c_ulonglong;
        let mut adata = adata.read_lock();
        let mut k = SecBuf::with_secure(32);
        let mut k = k.read_lock();
        let mut nonce = nonce.read_lock();
        let mut decrypted_message = decrypted_message.write_lock();
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_decrypt(raw_ptr_char!(decrypted_message),std::ptr::null_mut(),raw_ptr_char!(secret),raw_ptr_char_immut!(cipher),cipher_len,raw_ptr_char_immut!(adata),ad_len,raw_ptr_char_immut!(nonce),raw_ptr_char_immut!(k));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_return_Error_with_bad_cipher() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        let mut nonce = SecBuf::with_secure(32);
        let mut cipher = SecBuf::with_insecure(2);
        buf(&mut message);
        {
            let mut message = message.write_lock();
            let output  = enc(&mut message,&mut secret,None,&mut nonce,&mut cipher);
            match output{
                Ok(k)=>{
                    assert!(false)
                }
                Err(e)=>{
                    assert!(true)
                }
            }
        }
    }
    #[test]
    fn it_should_with_auth_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        buf(&mut secret);
        let mut nonce = SecBuf::with_secure(32);
        let mut cipher = SecBuf::with_secure(32);
        let mut adata = SecBuf::with_secure(16);
        buf(&mut message);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher);
        }
        let mut decrypted_message = SecBuf::with_secure(16);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!(format!("{:?}", *message), format!("{:?}", *decrypted_message));
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
            enc(&mut message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher);
        }
        let mut decrypted_message = SecBuf::with_secure(16);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut secret,Some(&mut adata1),&mut nonce,&mut cipher);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!("[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", format!("{:?}", *decrypted_message));
    }
    #[test]
    fn it_should_with_NONE_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        let mut nonce = SecBuf::with_secure(32);
        let mut cipher = SecBuf::with_secure(32);
        buf(&mut message);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut secret,None,&mut nonce,&mut cipher);
        }
        let mut decrypted_message = SecBuf::with_secure(16);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut secret,None,&mut nonce,&mut cipher);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!(format!("{:?}", *message), format!("{:?}", *decrypted_message));
    }
}
