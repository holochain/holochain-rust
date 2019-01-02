//! This module provides access to libsodium
use super::{check_init, secbuf::SecBuf};
use crate::error::SodiumResult;

/// Generate a signing keypair from a seed buffer
///
/// @param {SecBuf} publicKey - Empty Buffer to be used as publicKey return
///
/// @param {SecBuf} privateKey - Empty Buffer to be used as secretKey return
///
/// @param {SecBuf} seed - the seed to derive a keypair from
///
/// @UseReturn {SecBuf} - { publicKey, privateKey }
pub fn seed_keypair(
    public_key: &mut SecBuf,
    secret_key: &mut SecBuf,
    seed: &mut SecBuf,
) -> SodiumResult<()> {
    check_init();
    let seed = seed.read_lock();
    let mut secret_key = secret_key.write_lock();
    let mut public_key = public_key.write_lock();
    unsafe {
        rust_sodium_sys::crypto_sign_seed_keypair(
            raw_ptr_char!(public_key),
            raw_ptr_char!(secret_key),
            raw_ptr_char_immut!(seed),
        );
    }
    Ok(())
}

/// generate a signature
///
/// @param {Buffer} message - the message to sign
///
/// @param {SecBuf} secretKey - the secret key to sign with
///
/// @param {SecBuf} signature - Empty Buffer to be used as signature return
///
/// @UseReturn {SecBuf} {signature}
pub fn sign(
    message: &mut SecBuf,
    secret_key: &mut SecBuf,
    signature: &mut SecBuf,
) -> SodiumResult<()> {
    check_init();
    let message = message.read_lock();
    let secret_key = secret_key.read_lock();
    let mut signature = signature.write_lock();
    let mess_len = message.len() as libc::c_ulonglong;
    unsafe {
        rust_sodium_sys::crypto_sign_detached(
            raw_ptr_char!(signature),
            std::ptr::null_mut(),
            raw_ptr_char_immut!(message),
            mess_len,
            raw_ptr_char_immut!(secret_key),
        );
    }
    Ok(())
}

/// verify a signature given the message and a publicKey
///
/// @param {Buffer} signature
///
/// @param {Buffer} message
///
/// @param {Buffer} publicKey
pub fn verify(signature: &mut SecBuf, message: &mut SecBuf, public_key: &mut SecBuf) -> i32 {
    check_init();
    let signature = signature.read_lock();
    let message = message.read_lock();
    let public_key = public_key.read_lock();
    let mess_len = message.len() as libc::c_ulonglong;
    unsafe {
        return rust_sodium_sys::crypto_sign_verify_detached(
            raw_ptr_char_immut!(signature),
            raw_ptr_char_immut!(message),
            mess_len,
            raw_ptr_char_immut!(public_key),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::random_secbuf;
    #[test]
    fn it_should_get_true_on_good_verify() {
        let mut seed = SecBuf::with_secure(32);
        let mut public_key = SecBuf::with_secure(32);
        let mut secret_key = SecBuf::with_secure(64);
        let mut signature = SecBuf::with_secure(64);

        random_secbuf(&mut seed);

        seed_keypair(&mut public_key, &mut secret_key, &mut seed).unwrap();

        let mut message = SecBuf::with_insecure(32);
        random_secbuf(&mut message);

        sign(&mut message, &mut secret_key, &mut signature).unwrap();
        {
            let ver = verify(&mut signature, &mut message, &mut public_key);
            assert_eq!(0, ver);
        }
    }

    #[test]
    fn it_should_get_false_on_bad_verify() {
        let mut seed = SecBuf::with_secure(32);
        let mut public_key = SecBuf::with_secure(32);
        let mut secret_key = SecBuf::with_secure(64);
        let mut signature = SecBuf::with_secure(64);

        random_secbuf(&mut seed);

        seed_keypair(&mut public_key, &mut secret_key, &mut seed).unwrap();

        let mut message = SecBuf::with_insecure(32);
        random_secbuf(&mut message);

        let mut fake_message = SecBuf::with_insecure(32);
        random_secbuf(&mut fake_message);

        sign(&mut message, &mut secret_key, &mut signature).unwrap();

        {
            let ver = verify(&mut signature, &mut fake_message, &mut public_key);
            assert_eq!(-1, ver);
        }
    }
}
