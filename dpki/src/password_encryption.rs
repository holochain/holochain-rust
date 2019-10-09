use crate::{utils::secbuf_from_array, SecBuf, CRYPTO};
use holochain_core_types::error::HcResult;
use serde_derive::{Deserialize, Serialize};

pub type OpsLimit = u64;
pub type MemLimit = usize;
pub type PwHashAlgo = i8;

#[derive(Clone)]
pub struct PwHashConfig(pub OpsLimit, pub MemLimit, pub PwHashAlgo);

/// Struct holding the result of a passphrase encryption
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct EncryptedData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}

/// Simple API for generating a password hash with our set parameters
/// @param {SecBuf} password - the password buffer to hash
/// @param {SecBuf} salt - if specified, hash with this salt (otherwise random)
/// @param {SecBuf} hash_result - Empty SecBuf to receive the resulting hash.
/// @param {Option<PwHashConfig>} config - Optional hashing settings
/// TODO make salt optional
pub(crate) fn pw_hash(
    hash_result: &mut SecBuf,
    password: &mut SecBuf,
    salt: &mut SecBuf,
) -> HcResult<()> {
    CRYPTO.pwhash(hash_result, password, salt)?;
    Ok(())
}

/// Simple API for encrypting a buffer with a pwhash-ed passphrase
/// @param {Buffer} data - the data to encrypt
/// @param {SecBuf} passphrase - the passphrase to use for encrypting
/// @return {EncryptedData} - the resulting encrypted data
pub(crate) fn pw_enc(data: &mut SecBuf, passphrase: &mut SecBuf) -> HcResult<EncryptedData> {
    let mut salt = CRYPTO.buf_new_insecure(CRYPTO.pwhash_salt_bytes());
    CRYPTO.randombytes_buf(&mut salt)?;
    let mut nonce = CRYPTO.buf_new_insecure(CRYPTO.aead_nonce_bytes());
    CRYPTO.randombytes_buf(&mut nonce)?;
    pw_enc_base(data, passphrase, &mut salt, &mut nonce)
}

pub(crate) fn pw_enc_base(data: &mut SecBuf, passphrase: &mut SecBuf, mut salt: &mut SecBuf, mut nonce: &mut SecBuf) -> HcResult<EncryptedData> {
    let mut secret = CRYPTO.buf_new_secure(CRYPTO.kx_session_key_bytes());
    let mut cipher = CRYPTO.buf_new_insecure(data.len() + CRYPTO.aead_auth_bytes());
    pw_hash(&mut secret, passphrase, &mut salt)?;
    CRYPTO.aead_encrypt(&mut cipher, data, None, &mut nonce, &mut secret)?;
    // aead_encrypt!(CRYPTO =>
    //               cipher: &mut cipher,
    //               message: data,
    //               adata: None,
    //               nonce: &mut nonce,
    //               secret: &mut secret)?;

    let salt = salt.read_lock().to_vec();
    let nonce = nonce.read_lock().to_vec();
    let cipher = cipher.read_lock().to_vec();
    // Done
    Ok(EncryptedData {
        salt,
        nonce,
        cipher,
    })
}

/// Simple API for encrypting a buffer with a pwhash-ed passphrase but uses a zero nonce
/// This does not weaken security provided the same passphrase/salt is not used to encrypt multiple
/// pieces of data. Since a random salt is produced by this function it should not be an issue.
///  Helpful for reducing the size of the output EncryptedData (by NONCEBYTES)
/// @param {Buffer} data - the data to encrypt
/// @param {SecBuf} passphrase - the passphrase to use for encrypting
/// @param {Option<PwHashConfig>} config - Optional encrypting settings
/// @return {EncryptedData} - the resulting encrypted data
pub(crate) fn pw_enc_zero_nonce(
    data: &mut SecBuf,
    passphrase: &mut SecBuf,
) -> HcResult<EncryptedData> {
    let mut salt = CRYPTO.buf_new_insecure(CRYPTO.pwhash_salt_bytes());
    CRYPTO.randombytes_buf(&mut salt)?;
    let mut nonce = CRYPTO.buf_new_insecure(CRYPTO.aead_nonce_bytes());
    let len = CRYPTO.aead_nonce_bytes();
    let slice = vec![0; len];
    nonce.write(0, &slice )?;
    let data = pw_enc_base(data, passphrase, &mut salt, &mut nonce)?;
    Ok(data)
}

/// Simple API for decrypting a buffer with a pwhash-ed passphrase
/// @param {EncryptedData} encrypted_data - the data to decrypt
/// @param {SecBuf} passphrase - the passphrase to use for encrypting
/// @param {SecBuf} decrypted_data - the dresulting ecrypted data
pub(crate) fn pw_dec(
    encrypted_data: &EncryptedData,
    passphrase: &mut SecBuf,
    decrypted_data: &mut SecBuf,
) -> HcResult<()> {
    let mut secret = CRYPTO.buf_new_secure(CRYPTO.kx_session_key_bytes());
    let mut salt = CRYPTO.buf_new_insecure(CRYPTO.pwhash_salt_bytes());
    secbuf_from_array(&mut salt, &encrypted_data.salt)?;
    let mut nonce = CRYPTO.buf_new_insecure(encrypted_data.nonce.len());
    secbuf_from_array(&mut nonce, &encrypted_data.nonce)?;
    let mut cipher = CRYPTO.buf_new_insecure(encrypted_data.cipher.len());
    secbuf_from_array(&mut cipher, &encrypted_data.cipher)?;
    pw_hash(&mut secret, passphrase, &mut salt)?;
    CRYPTO.aead_decrypt(decrypted_data, &mut cipher, None, &mut nonce, &mut secret)?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::utils::tests::TEST_CRYPTO;

    fn test_password() -> SecBuf {
        let mut password = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.pwhash_bytes());
        {
            let mut password = password.write_lock();
            password[0] = 42;
            password[1] = 222;
        }
        password
    }

    #[test]
    fn it_should_encrypt_data() {
        let mut password = test_password();
        let mut data = TEST_CRYPTO.buf_new_insecure(32);
        {
            let mut data = data.write_lock();
            data[0] = 88;
            data[1] = 101;
        }
        let encrypted_data = pw_enc(&mut data, &mut password).unwrap();

        let mut decrypted_data = TEST_CRYPTO.buf_new_insecure(32);
        pw_dec(&encrypted_data, &mut password, &mut decrypted_data).unwrap();

        let data = data.read_lock();
        let decrypted_data = decrypted_data.read_lock();
        assert_eq!(format!("{:?}", decrypted_data), format!("{:?}", data));
    }

    #[test]
    fn it_should_generate_pw_hash_with_salt() {
        let mut password = test_password();
        let mut salt = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.pwhash_salt_bytes());
        let mut hashed_password = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.pwhash_bytes());
        pw_hash(&mut hashed_password, &mut password, &mut salt).unwrap();
        println!("salt = {:?}", salt);
        {
            let pw2_hash = hashed_password.read_lock();
            assert_eq!(
                "[134, 156, 170, 171, 184, 19, 40, 158, 64, 227, 105, 252, 59, 175, 119, 226, 77, 238, 49, 61, 27, 174, 47, 246, 179, 168, 88, 200, 65, 11, 14, 159]",
                format!("{:?}", pw2_hash),
            );
        }
        // hash with different salt should have different result
        TEST_CRYPTO.randombytes_buf(&mut salt).expect("should work");
        let mut hashed_password_b = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.pwhash_bytes());
        pw_hash(&mut hashed_password_b, &mut password, &mut salt).unwrap();
        assert!(hashed_password.compare(&mut hashed_password_b) != 0);

        // same hash should have same result
        let mut hashed_password_c = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.pwhash_bytes());
        pw_hash( &mut hashed_password_c, &mut password, &mut salt).unwrap();
        assert!(hashed_password_c.compare(&mut hashed_password_b) == 0);
    }

}
