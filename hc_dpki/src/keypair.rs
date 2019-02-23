#![allow(warnings)]
extern crate holochain_sodium;
use crate::keypair::holochain_sodium::{kx, secbuf::SecBuf, sign};
use holochain_sodium::random::random_secbuf;
use holochain_sodium::secbuf::SecBuf;
// use holochain_sodium::sign::*;
use hcid::*;

use crate::{
    key_bundle,
    util::{self, PwHashConfig},
};
use holochain_core_types::error::{HcResult, HolochainError};
use rustc_serialize::json;
use std::str;


pub const SEED_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;

pub type Base32 = String;

fn decode_pub_key_into_secbuf(pub_key_b32: &str, codec: &HcidEncoding) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key =     codec.decode(pub_key_b32)?;
    // convert to SecBuf
    let mut pub_key_sec = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    let mut pub_key_lock = pub_key_sec.write_lock();
    for x in 0..pub_key.len() {
        pub_key_lock[x] = pub_key[x];
    }
    Ok(pub_key_sec)
}

fn encode_pub_key(pub_key_sec: &mut SecBuf, codec: &HcidEncoding) -> HcResult<Base32> {
    let locker = pub_key_sec.read_lock();
    let pub_buf = &*locker;
    let pub_key = array_ref![pub_buf, 0, SEED_SIZE];
    Ok(codec.encode(pub_key.as_bytes())?)
}

pub trait KeyPairable {
    fn public(&self) -> Base32;
    // fn private(&self) -> SecBuf;

    fn new_from_seed(seed: &mut SecBuf);

    fn codec() -> HcidEncoding;

    /// Decode the public key from Base32 into a [u8]
    fn decode_pub_key(&self) -> Vec<u8> {
        let codec = self.codec();
        self.codec().decode().expect("Public key decoding failed. Key was not properly encoded.")
    }

    fn decode_pub_key_into_secbuf(&self) -> SecBuf {
        decode_pub_key_into_secbuf(&self.public(), &self.codec())
            .expect("Public key decoding failed. Key was not properly encoded.")
    }
}


//fn decode_pub_key_into_secbuf(codec: HcidEncoding, pub_key_b32: Base32) -> HcResult<SecBuf> {
//    // Decode Base32 public key
//    let pub_key = codec.decode(client_b32)?;
//    // convert to SecBuf
//    let mut pub_key_sec = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
//    let mut pub_key_lock = pub_key_sec.write_lock();
//    for x in 0..pub_key.len() {
//        pub_key_lock[x] = pub_key[x];
//    }
//    Ok(pub_key_sec)
//}

//pub enum KeyPairKind {
//    SIGNING,
//    ENCRYPTING,
//}

pub struct KeyPair {
    public: Base32,
    private: SecBuf,
}

impl KeyPair {
    pub fn new(public: Base32, private: SecBuf) -> Self {
        KeyPair { public, private }
    }
}

//--------------------------------------------------------------------------------------------------
// Signing KeyPair
//--------------------------------------------------------------------------------------------------

pub struct SigningKeyPair {
    keypair: KeyPair,
}

impl KeyPairable for SigningKeyPair {
    fn public(&self) -> String { self.keypair.public.clone() }
    // fn private(&self) -> SecBuf { self.keypair.private.clone() }

    fn codec() -> HcidEncoding {
        with_hcs0().expect("HCID failed miserably with_hcs0.")
    }

    /// derive the signing pair from a 32 byte seed buffer
    fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut priv_sec_buf = SecBuf::with_secure(sign::SECRETKEYBYTES);
        holochain_sodium::sign::seed_keypair(&mut pub_sec_buf, &mut priv_sec_buf, seed)?;
        // Convert and encode public key side
        let pub_key_b32 = encode_pub_key(&mut pub_sec_buf, &Self::codec())?;
        // Done
        Ok(SigningKeyPair::new(pub_key_b32, priv_sec_buf))
    }
}

impl SigningKeyPair {

    pub fn new(public: Base32, private: SecBuf) -> Self {
        SigningKeyPair { keypair: KeyPair::new(public, private) }
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
        let res= holochain_sodium::sign::verify(signature, data, &mut pub_key);
        res == 0
    }
}

//--------------------------------------------------------------------------------------------------
// Encrypting KeyPair
//--------------------------------------------------------------------------------------------------

pub struct EncryptingKeyPair {
    keypair: KeyPair,
}

impl EncryptingKeyPair {
    pub fn new(public: String, private: SecBuf) -> Self {
        EncryptingKeyPair { keypair: KeyPair::new(public, private) }
    }
}

impl KeyPairable for EncryptingKeyPair {
    fn public(&self) -> String { self.keypair.public.clone() }
    // fn private(&self) -> SecBuf { self.keypair.private.clone() }

    fn codec() -> HcidEncoding {
        with_hck0().expect("HCID failed miserably with_hck0.")
    }

    /// derive the signing pair from a 32 byte seed buffer
    fn new_from_seed(seed: &mut SecBuf) -> HcResult<Self> {
        assert_eq!(seed.len(), SEED_SIZE);
        // Generate keys
        let mut pub_sec_buf = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
        let mut priv_sec_buf = SecBuf::with_secure(kx::SECRETKEYBYTES);
        holochain_sodium::kx::seed_keypair(&mut pub_sec_buf, &mut priv_sec_buf, seed)?;
        // Convert and encode public key side
        let pub_key_b32 = encode_pub_key(&mut pub_sec_buf, &Self::codec())?;
        // Done
        Ok(EncryptingKeyPair::new(pub_key_b32, priv_sec_buf))
    }
}

impl EncryptingKeyPair {
    /// Decode the public key from Base32 into a [u8]
    fn decode_pub_key(&self) -> Vec<u8> {
        let codec = with_hck0().expect("HCID failed miserably.");
        codec.decode(&self.keypair.public).expect("Encrypting key decoding failed. Key was not properly encoded.")
    }
}

//--------------------------------------------------------------------------------------------------
// Test
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

//    #[test]
//    fn it_should_set_keypair_from_seed() {}
}