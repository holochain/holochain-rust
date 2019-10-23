#![allow(warnings)]

use crate::{
    key_bundle,
    password_encryption::{self, PwHashConfig},
    utils, SecBuf, CODEC_HCK0, CODEC_HCS0, CRYPTO, SEED_SIZE, SIGNATURE_SIZE,
};
use hcid::*;
use holochain_core_types::{agent::Base32, error::HcResult};
use serde_json::json;
use std::str;

pub trait KeyPair {
    // -- Interface to implement -- //

    fn public(&self) -> Base32;
    fn private(&mut self) -> &mut SecBuf;
    fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self>
    where
        Self: Sized;

    fn new_from_self(&mut self) -> HcResult<Self>
    where
        Self: Sized;

    fn codec<'a>() -> &'a HcidEncoding;

    // -- Common methods -- //

    /// Decode the public key from Base32 into a [u8]
    fn encode_pub_key(pub_key_sec: &mut SecBuf) -> Base32 {
        utils::encode_pub_key(pub_key_sec, &Self::codec()).expect("Public key encoding failed.")
    }

    /// Decode the public key from Base32 into a [u8]
    fn decode_pub_key(&self) -> Vec<u8> {
        Self::codec()
            .decode(&self.public())
            .expect("Public key decoding failed. Key was not properly encoded.")
    }

    /// Decode the public key from Base32 into a SecBuf
    fn decode_pub_key_into_secbuf(&self) -> SecBuf {
        utils::decode_pub_key(self.public(), Self::codec())
            .expect("Public key decoding failed. Key was not properly encoded.")
    }

    /// Return true if the keys are equivalent
    fn is_same(&mut self, other: &mut Self) -> bool {
        self.public() == other.public() && self.private().compare(other.private()) == 0
    }
}

//--------------------------------------------------------------------------------------------------
// Signing KeyPair
//--------------------------------------------------------------------------------------------------

/// KeyPair used for signing data
pub struct SigningKeyPair {
    pub public: Base32,
    pub private: SecBuf,
}

impl KeyPair for SigningKeyPair {
    fn public(&self) -> String {
        self.public.clone()
    }
    fn private(&mut self) -> &mut SecBuf {
        &mut self.private
    }
    fn codec<'a>() -> &'a HcidEncoding {
        &CODEC_HCS0
    }

    /// derive the signing pair from a 32 byte seed buffer
    fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = CRYPTO.buf_new_insecure(CRYPTO.sign_secret_key_bytes());
        let mut priv_sec_buf = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        CRYPTO.sign_seed_keypair(seed, &mut pub_sec_buf, &mut priv_sec_buf)?;
        // Convert and encode public key side
        let pub_key_b32 = utils::encode_pub_key(&mut pub_sec_buf, Self::codec())?;
        // Done
        Ok(SigningKeyPair::new(pub_key_b32, priv_sec_buf))
    }

    fn new_from_self(&mut self) -> HcResult<Self> {
        Ok(SigningKeyPair::new(self.public(), self.private.box_clone()))
    }
}

impl SigningKeyPair {
    /// Standard Constructor
    pub fn new(public: Base32, private: SecBuf) -> Self {
        Self { public, private }
    }

    /// Construct with a public key not already HCID encoded
    pub fn new_with_raw_key(pub_key_sec: &mut SecBuf, private: SecBuf) -> Self {
        let public = Self::encode_pub_key(pub_key_sec);
        Self { public, private }
    }

    /// sign some arbitrary data with the signing private key
    /// @param {SecBuf} data - the data to sign
    /// @return {SecBuf} signature - Empty SecBuf to be filled with the signature
    pub fn sign(&mut self, data: &mut SecBuf) -> HcResult<SecBuf> {
        let mut signature = CRYPTO.buf_new_insecure(SIGNATURE_SIZE);
        CRYPTO.sign(&mut signature, data, &mut self.private)?;
        Ok(signature)
    }

    /// verify data that was signed with our private signing key
    /// @param {SecBuf} data
    /// @param {SecBuf} signature
    /// @return true if verification succeeded
    pub fn verify(&mut self, data: &mut SecBuf, signature: &mut SecBuf) -> HcResult<bool> {
        let mut pub_key = self.decode_pub_key_into_secbuf();
        let result = CRYPTO.sign_verify(signature, data, &mut pub_key)?;
        Ok(result)
    }
}

//--------------------------------------------------------------------------------------------------
// Encrypting KeyPair
//--------------------------------------------------------------------------------------------------

/// KeyPair used for encrypting data
pub struct EncryptingKeyPair {
    pub public: Base32,
    pub private: SecBuf,
}

impl KeyPair for EncryptingKeyPair {
    fn public(&self) -> String {
        self.public.clone()
    }
    fn private(&mut self) -> &mut SecBuf {
        &mut self.private
    }

    fn codec<'a>() -> &'a HcidEncoding {
        &CODEC_HCK0
    }

    /// Derive the signing pair from a 32 byte seed buffer
    fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = CRYPTO.buf_new_insecure(CRYPTO.sign_public_key_bytes());
        let mut priv_sec_buf = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        CRYPTO.kx_seed_keypair(&mut pub_sec_buf, &mut priv_sec_buf, seed)?;
        // Convert and encode public key side
        let pub_key_b32 = utils::encode_pub_key(&mut pub_sec_buf, Self::codec())?;
        // Done
        Ok(EncryptingKeyPair::new(pub_key_b32, priv_sec_buf))
    }

    fn new_from_self(&mut self) -> HcResult<Self> {
        Ok(EncryptingKeyPair::new(self.public(), self.private.box_clone()))
    }
}

impl EncryptingKeyPair {
    /// Standard Constructor
    pub fn new(public: String, private: SecBuf) -> Self {
        Self { public, private }
    }

    pub fn new_with_secbuf(pub_key_sec: &mut SecBuf, private: SecBuf) -> Self {
        let public = Self::encode_pub_key(pub_key_sec);
        Self { public, private }
    }

    /// encrypt some arbitrary data with the signing private key
    /// @param {SecBuf} data - the data to encrypt
    /// @param {output} encrypted_data - result of data encryption
    pub fn encrypt(&mut self, data: &mut SecBuf, mut encrypted_data: &mut SecBuf) -> HcResult<()> {
        let mut nonce = CRYPTO.buf_new_insecure(CRYPTO.aead_nonce_bytes());
        CRYPTO.randombytes_buf(&mut nonce);

        //data to represent encryption data length
        let cipher_length = data.len() + CRYPTO.aead_auth_bytes();
        let mut cipher = CRYPTO.buf_new_insecure(cipher_length.clone());

        //data is encrypted and cipher is populated
        CRYPTO.aead_encrypt(&mut cipher, data, None, &mut nonce, &mut self.private)?;

        //get read locks from cipher
        let cipher_slice = &*cipher.read_lock();
        let nonce_slice = &*nonce.read_lock();

        //append nonce to cipher
        let cipher_with_nonce_slice = cipher_slice
            .into_iter()
            .cloned()
            .chain(nonce_slice.into_iter().cloned())
            .collect::<Vec<u8>>();
        utils::secbuf_from_array(&mut encrypted_data, &cipher_with_nonce_slice);

        Ok(())
    }

    /// decrypt some arbitrary data with the signing private key
    /// @param {SecBuf} cipher - the data to decrypt
    /// @param{SecBuf} data - the decrypted data
    pub fn decrypt(
        &mut self,
        cipher: &mut SecBuf,
        mut decrypted_message: &mut SecBuf,
    ) -> HcResult<()> {
        let cipher_length = cipher.len() - CRYPTO.aead_nonce_bytes();

        //get nonce from buffer
        let mut cipher_slice = &*cipher.read_lock();
        let mut nonce = CRYPTO.buf_new_insecure(CRYPTO.aead_nonce_bytes());
        let nonce_slice_from_cipher = cipher_slice
            .iter()
            .skip(cipher_length)
            .cloned()
            .collect::<Vec<u8>>();
        utils::secbuf_from_array(&mut nonce,&nonce_slice_from_cipher)?;

        //get cipher only from buffer
        let cipher_no_nonce_slice = cipher_slice
            .iter()
            .cloned()
            .take(cipher_length)
            .collect::<Vec<u8>>();
        let mut cipher_no_nonce = CRYPTO.buf_new_insecure(cipher_length);
        utils::secbuf_from_array(&mut cipher_no_nonce,&cipher_no_nonce_slice)?;

        CRYPTO.aead_decrypt(
            &mut decrypted_message,
            &mut cipher_no_nonce,
            None,
            &mut nonce,
            &mut self.private,
        )?;
        Ok(())
    }
}

pub fn generate_random_sign_keypair() -> HcResult<SigningKeyPair> {
    let mut seed = utils::generate_random_seed_buf();
    SigningKeyPair::new_from_seed(&mut seed)
}

pub fn generate_random_enc_keypair() -> HcResult<EncryptingKeyPair> {
    let mut seed = utils::generate_random_seed_buf();
    EncryptingKeyPair::new_from_seed(&mut seed)
}

//--------------------------------------------------------------------------------------------------
// Test
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SEED_SIZE;
    use crate::utils::tests::TEST_CRYPTO;

    pub fn test_generate_random_sign_keypair() -> SigningKeyPair {
        generate_random_sign_keypair().unwrap()
    }

    pub fn test_generate_random_enc_keypair() -> EncryptingKeyPair {
        generate_random_enc_keypair().unwrap()
    }

    #[test]
    fn keypair_should_construct_and_clone_sign() {
        let mut keys = test_generate_random_sign_keypair();
        // Test public key
        println!("sign_keys.public = {:?}", keys.public);
        assert_eq!(63, keys.public.len());
        let prefix: String = keys.public.chars().skip(0).take(3).collect();
        assert_eq!("HcS", prefix);
        // Test private key
        assert_eq!(64, keys.private.len());
        // Test clone
        assert!(keys.new_from_self().unwrap().is_same(&mut keys));
    }

    #[test]
    fn keypair_should_construct_and_clone_enc() {
        let mut keys = test_generate_random_enc_keypair();
        // Test public key
        println!("enc_keys.public = {:?}", keys.public);
        assert_eq!(63, keys.public.len());
        let prefix: String = keys.public.chars().skip(0).take(3).collect();
        assert_eq!("HcK", prefix);
        // Test private key
        assert_eq!(32, keys.private.len());
        // Test clone
        assert!(keys.new_from_self().unwrap().is_same(&mut keys));
    }

    #[test]
    fn keypair_should_sign_message_and_verify() {
        let mut sign_keys = test_generate_random_sign_keypair();

        // Create random data
        let mut message = TEST_CRYPTO.buf_new_insecure(16);
        TEST_CRYPTO.randombytes_buf(&mut message).expect("should work");

        // sign it
        let mut signature = sign_keys.sign(&mut message).unwrap();
        println!("signature = {:?}", signature);
        // authentify signature
        let succeeded = sign_keys.verify(&mut message, &mut signature);
        assert_eq!(succeeded, Ok(true));

        // Create random data
        let mut random_signature = TEST_CRYPTO.buf_new_insecure(SIGNATURE_SIZE);
        TEST_CRYPTO.randombytes_buf(&mut random_signature).expect("should work");
        // authentify random signature
        let succeeded = sign_keys.verify(&mut message, &mut random_signature);
        assert_eq!(succeeded, Ok(false));

        // Randomize data again
        TEST_CRYPTO.randombytes_buf(&mut message).expect("should work");
        let succeeded = sign_keys.verify(&mut message, &mut signature);
        assert_eq!(succeeded, Ok(false));
    }

}
