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
    let mut my_adata_locker;
    let mut my_adata = std::ptr::null();
    let mut my_ad_len = 0 as libc::c_ulonglong;

    if let Some(s) = adata {
        my_adata_locker = s.read_lock();
        my_adata = raw_ptr_char_immut!(my_adata_locker);
        my_ad_len = my_adata_locker.len() as libc::c_ulonglong;
    }

    let mut cipher = cipher.write_lock();
    let message = message.read_lock();
    let nonce = nonce.read_lock();
    let secret = secret.read_lock();

    unsafe {
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_encrypt(
            raw_ptr_char!(cipher),
            std::ptr::null_mut(),
            raw_ptr_char_immut!(message),
            message.len() as libc::c_ulonglong,
            my_adata,
            my_ad_len,
            std::ptr::null_mut(),
            raw_ptr_char_immut!(nonce),
            raw_ptr_char_immut!(secret)
        );
    }

    Ok(())
}

/// Decrypt symmetric cipher text given a nonce, secret, and optional auth data
/// ****
/// @param {SecBuf} decrypted_message - Empty Buffer to be used as output to return the result
/// @param {SecBuf} secret - symmetric secret key
/// @param {Buffer} adata - optional additional authenticated data
/// @param {Buffer} nonce - sometimes called initialization vector (iv)
/// @param {Buffer} cipher - the cipher text

pub fn dec(decrypted_message: &mut SecBuf,secret: &mut SecBuf,adata: Option<&mut SecBuf>,nonce: &mut SecBuf,cipher: &mut SecBuf)->SodiumResult<()>{
    let mut my_adata_locker;
    let mut my_adata = std::ptr::null();
    let mut my_ad_len = 0 as libc::c_ulonglong;

    if let Some(s) = adata {
        my_adata_locker = s.read_lock();
        my_adata = raw_ptr_char_immut!(my_adata_locker);
        my_ad_len = my_adata_locker.len() as libc::c_ulonglong;
    }

    let mut decrypted_message = decrypted_message.write_lock();
    let cipher = cipher.read_lock();
    let nonce = nonce.read_lock();
    let secret = secret.read_lock();

    unsafe {
        rust_sodium_sys::crypto_aead_xchacha20poly1305_ietf_decrypt(
            raw_ptr_char!(decrypted_message),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            raw_ptr_char_immut!(cipher),
            cipher.len() as libc::c_ulonglong,
            my_adata,
            my_ad_len,
            raw_ptr_char_immut!(nonce),
            raw_ptr_char_immut!(secret)
        );
    }


    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_with_auth_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(8);
        buf(&mut message);

        let mut secret = SecBuf::with_secure(32);
        buf(&mut secret);

        let mut adata = SecBuf::with_secure(16);
        buf(&mut adata);

        let mut nonce = SecBuf::with_insecure(16);
        buf(&mut nonce);

        let mut cipher = SecBuf::with_insecure(message.len() + ABYTES);

        enc(&mut message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher)
            .unwrap();

        let mut decrypted_message = SecBuf::with_insecure(
            cipher.len() - ABYTES);

        dec(&mut decrypted_message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher)
            .unwrap();

        {
            let message = message.read_lock();
            let decrypted_message = decrypted_message.read_lock();
            assert_eq!(
                format!("{:?}", *message),
                format!("{:?}", *decrypted_message));
        }
    }

    #[test]
    fn it_should_with_NONE_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        buf(&mut message);
        let mut message = message.write_lock();
        let cip_len = message.len() + ABYTES;
        let mut nonce = SecBuf::with_insecure(NONCEBYTES);
        let mut cipher = SecBuf::with_insecure(cip_len);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut secret,None,&mut nonce,&mut cipher);
        }
        let mut cipher = cipher.write_lock();
        let dec_len = cip_len - ABYTES;
        let mut decrypted_message = SecBuf::with_insecure(dec_len);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut secret,None,&mut nonce,&mut cipher);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!(format!("{:?}", *message), format!("{:?}", *decrypted_message));
    }
    #[test]
    fn it_should_with_bad_aead_encrypt_and_decrypt() {
        let mut message = SecBuf::with_secure(16);
        let mut secret = SecBuf::with_secure(32);
        let mut adata = SecBuf::with_secure(16);
        let mut adata1 = SecBuf::with_secure(16);
        buf(&mut adata1);
        buf(&mut message);
        let mut message = message.write_lock();
        let cip_len = message.len() + ABYTES;
        let mut nonce = SecBuf::with_insecure(NONCEBYTES);
        let mut cipher = SecBuf::with_insecure(cip_len);
        {
            let mut message = message.write_lock();
            enc(&mut message,&mut secret,Some(&mut adata),&mut nonce,&mut cipher);
        }
        let mut cipher = cipher.write_lock();
        let dec_len = cip_len - ABYTES;
        let mut decrypted_message = SecBuf::with_insecure(dec_len);
        {
            let mut decrypted_message = decrypted_message.write_lock();
            dec(&mut decrypted_message,&mut secret,Some(&mut adata1),&mut nonce,&mut cipher);
        }
        let mut message = message.read_lock();
        let mut decrypted_message = decrypted_message.read_lock();
        assert_eq!("[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]", format!("{:?}", *decrypted_message));
    }
}
