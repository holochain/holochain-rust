use crate::{
    keypair::{generate_random_sign_keypair, EncryptingKeyPair, KeyPair, SigningKeyPair},
    utils::{
        decrypt_with_passphrase_buf, encrypt_with_passphrase_buf, generate_derived_seed_buf,
        generate_random_buf, verify as signingkey_verify, SeedContext,
    },
    SEED_SIZE,
};
use holochain_core_types::{
    agent::Base32,
    cas::content::Address,
    error::{HcResult, HolochainError},
    signature::Signature,
};

use holochain_sodium::secbuf::SecBuf;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::collections::BTreeMap;
use crate::key_blob::KeyBlob;

pub const PCHECK_HEADER_SIZE: usize = 8;
pub const PCHECK_HEADER: [u8; 8] = *b"PHCCHECK";
pub const PCHECK_RANDOM_SIZE: usize = 32;
pub const PCHECK_SIZE: usize = PCHECK_RANDOM_SIZE + PCHECK_HEADER_SIZE;

pub enum Secret {
    SigningKey(SigningKeyPair),
    EncryptingKey(EncryptingKeyPair),
    Seed(SecBuf),
}

enum KeyType {
    Signing,
    Encrypting,
}

#[allow(dead_code)]
struct Keystore {
    passphrase_check: String,
    secrets: BTreeMap<String, KeyBlob>,
    #[serde(skip_serializing)]
    cache: HashMap<String, Arc<Mutex<Secret>>>,
}

fn make_passphrase_check(passphrase: &mut SecBuf) -> HcResult<String> {
    let mut check_buf = SecBuf::with_secure(PCHECK_SIZE);
    check_buf.randomize();
    check_buf.write(0, &PCHECK_HEADER).unwrap();
    encrypt_with_passphrase_buf(&mut check_buf, passphrase, None)
}

impl Keystore {
    #[allow(dead_code)]
    pub fn new(passphrase: &mut SecBuf) -> HcResult<Self> {
        Ok(Keystore {
            passphrase_check: make_passphrase_check(passphrase)?,
            secrets: BTreeMap::new(),
            cache: HashMap::new(),
        })
    }

    #[allow(dead_code)]
    fn check_passphrase(&self, mut passphrase: &mut SecBuf) -> HcResult<bool> {
        let mut decrypted_buf = decrypt_with_passphrase_buf(
            &self.passphrase_check,
            &mut passphrase,
            None,
            PCHECK_SIZE,
        )?;
        let mut decrypted_header = SecBuf::with_insecure(PCHECK_HEADER_SIZE);
        let decrypted_buf = decrypted_buf.read_lock();
        decrypted_header.write(0, &decrypted_buf[0..PCHECK_HEADER_SIZE])?;
        let mut expected_header = SecBuf::with_secure(PCHECK_HEADER_SIZE);
        expected_header.write(0, &PCHECK_HEADER)?;
        Ok(decrypted_header.compare(&mut expected_header) == 0)
    }

    #[allow(dead_code)]
    pub fn change_passphrase(
        &mut self,
        old_passphrase: &mut SecBuf,
        new_passphrase: &mut SecBuf,
    ) -> HcResult<()> {
        if !self.check_passphrase(old_passphrase)? {
            return Err(HolochainError::ErrorGeneric("Bad passphrase".to_string()));
        }
        self.passphrase_check = make_passphrase_check(new_passphrase)?;
        Ok(())
    }

    /// return a list of the identifiers stored in the keystore
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<String> {
        self.cache.keys().map(|k| k.to_string()).collect()
    }

    /// adds a secret to the keystore
    #[allow(dead_code)]
    pub fn add(&mut self, dst_id_str: &str, secret: Arc<Mutex<Secret>>) -> HcResult<()> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        self.cache.insert(dst_id, secret);
        Ok(())
    }

    /// adds a random root seed into the keystore
    #[allow(dead_code)]
    pub fn add_random_seed(&mut self, dst_id_str: &str, size: usize) -> HcResult<()> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        let seed_buf = generate_random_buf(size);
        let secret = Arc::new(Mutex::new(Secret::Seed(seed_buf)));
        self.cache.insert(dst_id, secret);
        Ok(())
    }

    fn check_dst_identifier(&self, dst_id_str: &str) -> HcResult<String> {
        let dst_id = dst_id_str.to_string();
        if self.cache.contains_key(&dst_id) {
            return Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string(),
            ));
        }
        Ok(dst_id)
    }

    /// gets a secret from the keystore
    #[allow(dead_code)]
    pub fn get(&self, src_id_str: &str) -> HcResult<Arc<Mutex<Secret>>> {
        let src_id = src_id_str.to_string();
        if !self.cache.contains_key(&src_id) {
            return Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string(),
            ));
        }
        Ok(self.cache.get(&src_id).unwrap().clone()) // unwrap ok because we checked if src exists
    }

    fn check_identifiers(
        &self,
        src_id_str: &str,
        dst_id_str: &str,
    ) -> HcResult<(Arc<Mutex<Secret>>, String)> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        let src_secret = self.get(src_id_str)?;
        Ok((src_secret, dst_id))
    }

    /// adds a derived seed into the keystore
    #[allow(dead_code)]
    pub fn add_seed_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_str: &str,
        context: &SeedContext,
        index: u64,
    ) -> HcResult<()> {
        let (src_secret, dst_id) = self.check_identifiers(src_id_str, dst_id_str)?;
        let secret = {
            let mut src_secret = src_secret.lock().unwrap();
            match *src_secret {
                Secret::Seed(ref mut src) => {
                    let seed = generate_derived_seed_buf(src, context, index, SEED_SIZE)?;
                    Arc::new(Mutex::new(Secret::Seed(seed)))
                }
                _ => {
                    return Err(HolochainError::ErrorGeneric(
                        "source secret is not a root seed".to_string(),
                    ));
                }
            }
        };
        self.cache.insert(dst_id, secret);

        Ok(())
    }

    /// adds a keypair into the keystore based on a seed already in the keystore
    /// returns the public key
    #[allow(dead_code)]
    fn add_key_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_str: &str,
        context: &SeedContext,
        index: u64,
        key_type: KeyType,
    ) -> HcResult<Base32> {
        let (src_secret, dst_id) = self.check_identifiers(src_id_str, dst_id_str)?;
        let (secret, public_key) = {
            let mut src_secret = src_secret.lock().unwrap();
            let ref mut seed = match *src_secret {
                Secret::Seed(ref mut src) => src,
                _ => {
                    return Err(HolochainError::ErrorGeneric(
                        "source secret is not a seed".to_string(),
                    ));
                }
            };
            let mut key_seed_buf = generate_derived_seed_buf(seed, context, index, SEED_SIZE)?;
            match key_type {
                KeyType::Signing => {
                    let key_pair = SigningKeyPair::new_from_seed(&mut key_seed_buf)?;
                    let public_key = key_pair.public();
                    (
                        Arc::new(Mutex::new(Secret::SigningKey(key_pair))),
                        public_key,
                    )
                }
                KeyType::Encrypting => {
                    let key_pair = EncryptingKeyPair::new_from_seed(&mut key_seed_buf)?;
                    let public_key = key_pair.public();
                    (
                        Arc::new(Mutex::new(Secret::EncryptingKey(key_pair))),
                        public_key,
                    )
                }
            }
        };
        self.cache.insert(dst_id, secret);

        Ok(public_key)
    }

    /// adds a signing keypair into the keystore based on a seed already in the keystore
    /// returns the public key
    #[allow(dead_code)]
    pub fn add_signing_key_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_str: &str,
        context: &SeedContext,
        index: u64,
    ) -> HcResult<Base32> {
        self.add_key_from_seed(src_id_str, dst_id_str, context, index, KeyType::Signing)
    }

    /// adds a signing keypair into the keystore based on a seed already in the keystore
    /// returns the public key
    #[allow(dead_code)]
    pub fn add_encrypting_key_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_str: &str,
        context: &SeedContext,
        index: u64,
    ) -> HcResult<Base32> {
        self.add_key_from_seed(src_id_str, dst_id_str, context, index, KeyType::Encrypting)
    }

    /// signs some data using a keypair in the keystore
    /// returns the signature
    #[allow(dead_code)]
    pub fn sign(&mut self, src_id_str: &str, data: String) -> HcResult<Signature> {
        let src_secret = self.get(src_id_str)?;
        let mut src_secret = src_secret.lock().unwrap();
        match *src_secret {
            Secret::SigningKey(ref mut key_pair) => {
                let mut data_buf = SecBuf::with_insecure_from_string(data);

                let mut signature_buf = key_pair.sign(&mut data_buf)?;
                let buf = signature_buf.read_lock();
                // Return as base64 encoded string
                let signature_str = base64::encode(&**buf);
                Ok(Signature::from(signature_str))
            }
            _ => {
                return Err(HolochainError::ErrorGeneric(
                    "source secret is not a signing key".to_string(),
                ));
            }
        }
    }
}

/// verifies data and signature against a public key
#[allow(dead_code)]
pub fn verify(public_key: Base32, data: String, signature: Signature) -> HcResult<bool> {
    signingkey_verify(Address::from(public_key), data, signature)
}

/// creates a one-time private key and sign data returning the signature and the public key
#[allow(dead_code)]
pub fn sign_one_time(data: String) -> HcResult<(Base32, Signature)> {
    let mut data_buf = SecBuf::with_insecure_from_string(data);
    let mut sign_keys = generate_random_sign_keypair()?;

    let mut signature_buf = sign_keys.sign(&mut data_buf)?;
    let buf = signature_buf.read_lock();
    // Return as base64 encoded string
    let signature_str = base64::encode(&**buf);
    Ok((sign_keys.public, Signature::from(signature_str)))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{utils, AGENT_ID_CTX};
    use base64;

    fn new_test_keystore() -> Keystore {
        let mut random_passphrase = utils::generate_random_buf(10);
        Keystore::new(&mut random_passphrase).unwrap()
    }

    #[test]
    fn test_keystore_new() {
        let mut random_passphrase = utils::generate_random_buf(10);
        let keystore = Keystore::new(&mut random_passphrase).unwrap();
        assert!(keystore.list().is_empty());
        assert_eq!(keystore.check_passphrase(&mut random_passphrase), Ok(true));
        let mut another_random_passphrase = utils::generate_random_buf(10);
        assert_eq!(
            keystore.check_passphrase(&mut another_random_passphrase),
            Ok(false)
        );
    }

    #[test]
    fn test_keystore_change_passphrase() {
        let mut random_passphrase = utils::generate_random_buf(10);
        let mut keystore = Keystore::new(&mut random_passphrase).unwrap();
        let mut another_random_passphrase = utils::generate_random_buf(10);
        assert!(
            // wrong passphrase
            keystore
                .change_passphrase(&mut another_random_passphrase, &mut random_passphrase)
                .is_err()
        );
        assert_eq!(
            keystore.change_passphrase(&mut random_passphrase, &mut another_random_passphrase),
            Ok(())
        );
        // check that passphrase was actually changed
        assert_eq!(keystore.check_passphrase(&mut random_passphrase), Ok(false));
        assert_eq!(
            keystore.check_passphrase(&mut another_random_passphrase),
            Ok(true)
        );
    }

    #[test]
    fn test_keystore_add_random_seed() {
        let mut keystore = new_test_keystore();

        assert_eq!(keystore.add_random_seed("my_root_seed", SEED_SIZE), Ok(()));
        assert_eq!(keystore.list(), vec!["my_root_seed".to_string()]);
        assert_eq!(
            keystore.add_random_seed("my_root_seed", SEED_SIZE),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_add_seed_from_seed() {
        let mut keystore = new_test_keystore();

        let context = SeedContext::new(*b"SOMECTXT");

        assert_eq!(
            keystore.add_seed_from_seed("my_root_seed", "my_second_seed", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        assert_eq!(
            keystore.add_seed_from_seed("my_root_seed", "my_second_seed", &context, 1),
            Ok(())
        );

        assert!(keystore.list().contains(&"my_root_seed".to_string()));
        assert!(keystore.list().contains(&"my_second_seed".to_string()));

        assert_eq!(
            keystore.add_seed_from_seed("my_root_seed", "my_second_seed", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_add_signing_key_from_seed() {
        let mut keystore = new_test_keystore();
        let context = SeedContext::new(AGENT_ID_CTX);

        assert_eq!(
            keystore.add_signing_key_from_seed("my_root_seed", "my_keypair", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        let result = keystore.add_signing_key_from_seed("my_root_seed", "my_keypair", &context, 1);
        assert!(!result.is_err());
        let pubkey = result.unwrap();
        assert!(format!("{}", pubkey).starts_with("Hc"));

        assert_eq!(
            keystore.add_signing_key_from_seed("my_root_seed", "my_keypair", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_sign() {
        let mut keystore = new_test_keystore();
        let context = SeedContext::new(AGENT_ID_CTX);

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        let data = base64::encode("the data to sign");

        assert_eq!(
            keystore.sign("my_keypair", data.clone()),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let public_key = keystore
            .add_signing_key_from_seed("my_root_seed", "my_keypair", &context, 1)
            .unwrap();

        let result = keystore.sign("my_keypair", data.clone());
        assert!(!result.is_err());

        let signature = result.unwrap();
        assert_eq!(String::from(signature.clone()).len(), 88); //88 is the size of a base64ized signature buf

        let result = verify(public_key, data.clone(), signature);
        assert!(!result.is_err());
        assert!(result.unwrap());

        keystore
            .add_encrypting_key_from_seed("my_root_seed", "my_enc_keypair", &context, 1)
            .unwrap();
        assert_eq!(
            keystore.sign("my_enc_keypair", data.clone()),
            Err(HolochainError::ErrorGeneric(
                "source secret is not a signing key".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_sign_one_time() {
        let data = base64::encode("the data to sign");
        let result = sign_one_time(data.clone());
        assert!(!result.is_err());

        let (public_key, signature) = result.unwrap();

        assert_eq!(String::from(signature.clone()).len(), 88); //88 is the size of a base64ized signature buf

        let result = verify(public_key, data, signature);
        assert!(!result.is_err());
        assert!(result.unwrap());
    }

}
