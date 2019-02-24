#![allow(warnings)]

use crate::{
    key_bundle,
    password_encryption::{self, PwHashConfig},
    utils, SEED_SIZE,
};
use hcid::*;
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
};
use holochain_sodium::{kx, secbuf::SecBuf, sign};
use rustc_serialize::json;
use std::str;

pub const SIGNATURE_SIZE: usize = 64;

pub trait KeyPairable {
    fn public(&self) -> Base32;
    // fn private(&self) -> SecBuf;

    // fn new_from_seed(seed: &mut SecBuf) -> HcResult<KeyPairable>;

    fn codec() -> HcidEncoding;

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

    fn decode_pub_key_into_secbuf(&self) -> SecBuf {
        utils::decode_pub_key_into_secbuf(&self.public(), &Self::codec())
            .expect("Public key decoding failed. Key was not properly encoded.")
    }

    fn create_secbuf() -> SecBuf;
}

pub struct KeyPair {
    pub public: Base32,
    pub private: SecBuf,
}

impl KeyPair {
    pub fn new(public: Base32, private: SecBuf) -> Self {
        KeyPair { public, private }
    }

    pub fn is_same(&mut self, other: &mut KeyPair) -> bool {
        self.public == other.public && self.private.dump() == other.private.dump()
    }
}

//--------------------------------------------------------------------------------------------------
// Signing KeyPair
//--------------------------------------------------------------------------------------------------

pub struct SigningKeyPair {
    pub keypair: KeyPair,
}

impl KeyPairable for SigningKeyPair {
    fn public(&self) -> String {
        self.keypair.public.clone()
    }
    // fn private(&self) -> SecBuf { self.keypair.private.clone() }

    fn codec() -> HcidEncoding {
        with_hcs0().expect("HCID failed miserably with_hcs0.")
    }

    fn create_secbuf() -> SecBuf {
        SecBuf::with_insecure(SIGNATURE_SIZE)
    }
}

impl SigningKeyPair {
    pub fn new(public: Base32, private: SecBuf) -> Self {
        SigningKeyPair {
            keypair: KeyPair::new(public, private),
        }
    }

    pub fn new_with_secbuf(pub_key_sec: &mut SecBuf, private: SecBuf) -> Self {
        let public = Self::encode_pub_key(pub_key_sec);
        Self {
            keypair: KeyPair::new(public, private),
        }
    }

    /// derive the signing pair from a 32 byte seed buffer
    pub fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut priv_sec_buf = SecBuf::with_secure(sign::SECRETKEYBYTES);
        holochain_sodium::sign::seed_keypair(&mut pub_sec_buf, &mut priv_sec_buf, seed)?;
        // Convert and encode public key side
        let pub_key_b32 = utils::encode_pub_key(&mut pub_sec_buf, &Self::codec())?;
        // Done
        Ok(SigningKeyPair::new(pub_key_b32, priv_sec_buf))
    }

    /// sign some arbitrary data with the signing private key
    /// @param {SecBuf} data - the data to sign
    /// @param {SecBuf} signature - Empty Buf to be filled with the signature
    pub fn sign(&mut self, data: &mut SecBuf, signature: &mut SecBuf) -> HcResult<()> {
        holochain_sodium::sign::sign(data, &mut self.keypair.private, signature)?;
        Ok(())
    }

    /// verify data that was signed with our private signing key
    /// @param {SecBuf} data
    /// @param {SecBuf} signature
    /// @return true if verification succeeded
    pub fn verify(&mut self, data: &mut SecBuf, signature: &mut SecBuf) -> bool {
        let mut pub_key = self.decode_pub_key_into_secbuf();
        let res = holochain_sodium::sign::verify(signature, data, &mut pub_key);
        res == 0
    }
}

//--------------------------------------------------------------------------------------------------
// Encrypting KeyPair
//--------------------------------------------------------------------------------------------------

pub struct EncryptingKeyPair {
    pub keypair: KeyPair,
}

impl KeyPairable for EncryptingKeyPair {
    fn public(&self) -> String {
        self.keypair.public.clone()
    }
    // fn private(&self) -> SecBuf { self.keypair.private.clone() }

    fn codec() -> HcidEncoding {
        with_hck0().expect("HCID failed miserably with_hck0.")
    }

    fn create_secbuf() -> SecBuf {
        SecBuf::with_secure(SEED_SIZE)
    }
}

impl EncryptingKeyPair {
    /// Standard Constructor
    pub fn new(public: String, private: SecBuf) -> Self {
        EncryptingKeyPair {
            keypair: KeyPair::new(public, private),
        }
    }

    pub fn new_with_secbuf(pub_key_sec: &mut SecBuf, private: SecBuf) -> Self {
        let public = Self::encode_pub_key(pub_key_sec);
        Self {
            keypair: KeyPair::new(public, private),
        }
    }

    /// Derive the signing pair from a 32 byte seed buffer
    pub fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
        let mut priv_sec_buf = SecBuf::with_secure(kx::SECRETKEYBYTES);
        holochain_sodium::kx::seed_keypair(&mut pub_sec_buf, &mut priv_sec_buf, seed)?;
        // Convert and encode public key side
        let pub_key_b32 = utils::encode_pub_key(&mut pub_sec_buf, &Self::codec())?;
        // Done
        Ok(EncryptingKeyPair::new(pub_key_b32, priv_sec_buf))
    }

    /// Decode the public key from Base32 into a [u8]
    pub fn decode_pub_key(&self) -> Vec<u8> {
        let codec = with_hck0().expect("HCID failed miserably.");
        codec
            .decode(&self.keypair.public)
            .expect("Encrypting key decoding failed. Key was not properly encoded.")
    }
}

//--------------------------------------------------------------------------------------------------
// Test
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_generate_random_seed() -> SecBuf {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        seed.randomize();
        seed
    }

    fn test_generate_random_sign_keypair() -> SigningKeyPair {
        let mut seed = test_generate_random_seed();
        SigningKeyPair::new_from_seed(&mut seed).unwrap()
    }

    fn test_generate_random_enc_keypair() -> EncryptingKeyPair {
        let mut seed = test_generate_random_seed();
        EncryptingKeyPair::new_from_seed(&mut seed).unwrap()
    }

    #[test]
    fn keypair_should_construct_sign() {
        let mut sign_keys = test_generate_random_sign_keypair();
        // Test public key
        println!("sign_keys.public = {:?}", sign_keys.keypair.public);
        assert_eq!(63, sign_keys.keypair.public.len());
        let prefix: String = sign_keys.keypair.public.chars().skip(0).take(3).collect();
        assert_eq!("HcS", prefix);
        // Test private key
        assert_eq!(64, sign_keys.keypair.private.len());
    }

    #[test]
    fn keypair_should_construct_enc() {
        let mut sign_keys = test_generate_random_enc_keypair();
        // Test public key
        println!("sign_keys.public = {:?}", sign_keys.keypair.public);
        assert_eq!(63, sign_keys.keypair.public.len());
        let prefix: String = sign_keys.keypair.public.chars().skip(0).take(3).collect();
        assert_eq!("HcK", prefix);
        // Test private key
        assert_eq!(32, sign_keys.keypair.private.len());
    }

    #[test]
    fn keypair_should_sign_message_and_verify() {
        let mut sign_keys = test_generate_random_sign_keypair();

        // Create random data
        let mut message = SecBuf::with_insecure(16);
        message.randomize();

        // sign it
        let mut signature = SigningKeyPair::create_secbuf();
        sign_keys.sign(&mut message, &mut signature).unwrap();
        println!("signature = {:?}", signature);
        // authentify signature
        let succeeded = sign_keys.verify(&mut message, &mut signature);
        assert!(succeeded);

        // Create random data
        let mut random_signature = SigningKeyPair::create_secbuf();
        random_signature.randomize();
        // authentify random signature
        let succeeded = sign_keys.verify(&mut message, &mut random_signature);
        assert!(!succeeded);

        // Randomize data again
        message.randomize();
        let succeeded = sign_keys.verify(&mut message, &mut signature);
        assert!(!succeeded);
    }
}
