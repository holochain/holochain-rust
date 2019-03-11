use crate::{
    key_blob::{Blobbable, KeyBlob},
    key_bundle::KeyBundle,
    seed::{generate_random_seed_buf, IndexedSeed, RootSeed, SeedContext, SeedTrait, SeedType},
    AGENT_ID_CTX_STR, SEED_SIZE,
};
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
    signature::Signature,
};

use holochain_sodium::{kdf, pwhash, secbuf::SecBuf};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub enum Secret {
    Key(KeyBundle),
    RootSeed(RootSeed),
    IndexedSeed(IndexedSeed),
}

struct KeyStore {
    keys: HashMap<String, Arc<Mutex<Secret>>>,
}

impl KeyStore {
    pub fn new() -> Self {
        KeyStore {
            keys: HashMap::new(),
        }
    }

    /// return a list of the identifiers stored in the keystore
    pub fn list(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for k in self.keys.keys() {
            ids.push(k.to_string());
        }
        ids
    }

    /// adds a random root seed into the keystore
    pub fn add_random_seed(&mut self, id_str: &str, size: usize) -> HcResult<()> {
        let id = id_str.to_string();
        if self.keys.contains_key(&id) {
            return Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string(),
            ));
        }
        let mut seed_buf = generate_random_seed_buf(size);
        let secret = Arc::new(Mutex::new(Secret::RootSeed(RootSeed::new(seed_buf))));
        let _ = self.keys.insert(id, secret);
        Ok(())
    }

    fn check_dst_identifier(&self, dst_id_str: &str) -> HcResult<String> {
        let dst_id = dst_id_str.to_string();
        if self.keys.contains_key(&dst_id) {
            return Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string(),
            ));
        }
        Ok(dst_id)
    }

    fn check_src_identifier(&self, src_id_str: &str) -> HcResult<Arc<Mutex<Secret>>> {
        let src_id = src_id_str.to_string();
        if !self.keys.contains_key(&src_id) {
            return Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string(),
            ));
        }
        Ok(self.keys.get(&src_id).unwrap().clone()) // unwrap ok because we checked if src exists
    }

    fn check_identifiers(
        &self,
        src_id_str: &str,
        dst_id_str: &str,
    ) -> HcResult<(Arc<Mutex<Secret>>, String)> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        let src_secret = self.check_src_identifier(src_id_str)?;
        Ok((src_secret, dst_id))
    }

    /// adds a derived seed into the keystore
    pub fn add_derived_seed(
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
                Secret::RootSeed(ref mut src) => {
                    let seed = src.generate_indexed_seed(context, index)?;
                    Arc::new(Mutex::new(Secret::IndexedSeed(seed)))
                }
                _ => {
                    return Err(HolochainError::ErrorGeneric(
                        "source secret is not a root seed".to_string(),
                    ));
                }
            }
        };
        let _ = self.keys.insert(dst_id, secret);

        Ok(())
    }

    /// adds a keypair into the keystore based on a seed already in the keystore
    /// returns the public key
    pub fn add_key_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_str: &str,
        context: &SeedContext,
        index: u64,
    ) -> HcResult<Base32> {
        let (src_secret, dst_id) = self.check_identifiers(src_id_str, dst_id_str)?;
        let (secret, public_key) = {
            let mut src_secret = src_secret.lock().unwrap();
            let ref mut seed = match *src_secret {
                Secret::RootSeed(ref mut src) => src.seed_mut(),
                Secret::IndexedSeed(ref mut src) => src.seed_mut(),
                _ => {
                    return Err(HolochainError::ErrorGeneric(
                        "source secret is not a seed".to_string(),
                    ));
                }
            };
            let mut key_seed_buf = SecBuf::with_secure(SEED_SIZE);
            let mut context = context.to_sec_buf();
            kdf::derive(&mut key_seed_buf, index, &mut context, &mut seed.buf)?;

            let key_bundle =
                KeyBundle::new_from_seed_buf(&mut key_seed_buf, SeedType::Application)?;
            let public_key = key_bundle.get_id();
            (Arc::new(Mutex::new(Secret::Key(key_bundle))), public_key)
        };
        let _ = self.keys.insert(dst_id, secret);

        Ok(public_key)
    }

    /// adds a keypair into the keystore based on a seed already in the keystore
    /// returns the public key
    pub fn sign(&mut self, src_id_str: &str, message: String) -> HcResult<Signature> {
        let src_secret = self.check_src_identifier(src_id_str)?;
        let mut src_secret = src_secret.lock().unwrap();
        match *src_secret {
            Secret::Key(ref mut key_bundle) => {
                let mut message_buf = SecBuf::with_insecure_from_string(message);

                let mut signature_buf = key_bundle.sign(&mut message_buf)?;
                let buf = signature_buf.read_lock();
                // Return as base64 encoded string
                let signature = base64::encode(&**buf);
                Ok(Signature::from(signature))
            }
            _ => {
                return Err(HolochainError::ErrorGeneric(
                    "source secret is not a key".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use base64;

    #[test]
    fn test_keystore_new() {
        let keystore = KeyStore::new();
        assert!(keystore.list().is_empty());
    }

    #[test]
    fn test_keystore_add_random_seed() {
        let mut keystore = KeyStore::new();

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
    fn test_keystore_add_derived_seed() {
        let mut keystore = KeyStore::new();

        let context = SeedContext::from("SOMECTXT");

        assert_eq!(
            keystore.add_derived_seed("my_root_seed", "my_second_seed", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        assert_eq!(
            keystore.add_derived_seed("my_root_seed", "my_second_seed", &context, 1),
            Ok(())
        );

        assert!(keystore.list().contains(&"my_root_seed".to_string()));
        assert!(keystore.list().contains(&"my_second_seed".to_string()));

        assert_eq!(
            keystore.add_derived_seed("my_root_seed", "my_second_seed", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_add_key_from_seed() {
        let mut keystore = KeyStore::new();
        let context = SeedContext::from(AGENT_ID_CTX_STR);

        assert_eq!(
            keystore.add_key_from_seed("my_root_seed", "my_keypair", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        let result = keystore.add_key_from_seed("my_root_seed", "my_keypair", &context, 1);
        assert!(!result.is_err());
        let pubkey = result.unwrap();
        assert!(format!("{}", pubkey).starts_with("Hc"));

        assert_eq!(
            keystore.add_key_from_seed("my_root_seed", "my_keypair", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );
    }

    #[test]
    fn test_keystore_sign() {
        let mut keystore = KeyStore::new();
        let context = SeedContext::from(AGENT_ID_CTX_STR);

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        let message = base64::encode("SOMEPAYLOAasdfasdfD");

        assert_eq!(
            keystore.sign("my_keypair", message.clone()),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_key_from_seed("my_root_seed", "my_keypair", &context, 1);

        let result = keystore.sign("my_keypair", message);
        assert!(!result.is_err());

        let signature = result.unwrap();
        assert_eq!(String::from(signature).len(), 88); //88 is the size of a base64ized signature buf
    }
}
