use holochain_core_types::{
    agent::Base32,
    cas::content::Address,
    error::{HcResult, HolochainError},
    signature::Signature,
};
use holochain_dpki::{
    key_blob::{BlobType, Blobbable, KeyBlob},
    key_bundle::KeyBundle,
    keypair::{generate_random_sign_keypair, EncryptingKeyPair, KeyPair, SigningKeyPair},
    seed::Seed,
    utils::{
        decrypt_with_passphrase_buf, encrypt_with_passphrase_buf, generate_derived_seed_buf,
        generate_random_buf, verify as signingkey_verify, SeedContext,
    },
    SEED_SIZE,
};

use holochain_sodium::secbuf::SecBuf;
use holochain_sodium::pwhash::{OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE, ALG_ARGON2ID13};

use conductor::passphrase_manager::PassphraseManager;
use holochain_dpki::seed::SeedType;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::prelude::*,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use holochain_dpki::password_encryption::PwHashConfig;

const PCHECK_HEADER_SIZE: usize = 8;
const PCHECK_HEADER: [u8; 8] = *b"PHCCHECK";
const PCHECK_RANDOM_SIZE: usize = 32;
const PCHECK_SIZE: usize = PCHECK_RANDOM_SIZE + PCHECK_HEADER_SIZE;
const KEYBUNDLE_SIGNKEY_SUFFIX: &str = ":sign_key";
const KEYBUNDLE_ENCKEY_SUFFIX: &str = ":enc_key";

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
#[derive(Serialize, Deserialize)]
pub struct Keystore {
    passphrase_check: String,
    secrets: BTreeMap<String, KeyBlob>,
    #[serde(skip_serializing, skip_deserializing)]
    cache: HashMap<String, Arc<Mutex<Secret>>>,
    #[serde(skip_serializing, skip_deserializing)]
    passphrase_manager: Option<Arc<PassphraseManager>>,
    #[serde(skip_serializing, skip_deserializing)]
    hash_config: Option<PwHashConfig>,
}

fn make_passphrase_check(passphrase: &mut SecBuf, hash_config: Option<PwHashConfig>) -> HcResult<String> {
    let mut check_buf = SecBuf::with_secure(PCHECK_SIZE);
    check_buf.randomize();
    check_buf.write(0, &PCHECK_HEADER).unwrap();
    encrypt_with_passphrase_buf(&mut check_buf, passphrase, hash_config)
}

impl Keystore {
    #[allow(dead_code)]
    pub fn new(passphrase_manager: Arc<PassphraseManager>, hash_config: Option<PwHashConfig>) -> HcResult<Self> {
        Ok(Keystore {
            passphrase_check: make_passphrase_check(&mut passphrase_manager.get_passphrase()?, hash_config.clone())?,
            secrets: BTreeMap::new(),
            cache: HashMap::new(),
            passphrase_manager: Some(passphrase_manager),
            hash_config,
        })
    }

    #[allow(dead_code)]
    pub fn new_from_file(
        path: PathBuf,
        passphrase_manager: Arc<PassphraseManager>,
        hash_config: Option<PwHashConfig>,
    ) -> HcResult<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut keystore: Keystore = serde_json::from_str(&contents)?;
        keystore.hash_config = hash_config;
        keystore.passphrase_manager = Some(passphrase_manager);
        Ok(keystore)
    }

    #[allow(dead_code)]
    fn check_passphrase(&self, mut passphrase: &mut SecBuf) -> HcResult<bool> {
        let mut decrypted_buf = decrypt_with_passphrase_buf(
            &self.passphrase_check,
            &mut passphrase,
            self.hash_config.clone(),
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
        self.passphrase_check = make_passphrase_check(new_passphrase, self.hash_config.clone())?;
        Ok(())
    }

    fn decrypt(&mut self, id_str: &String) -> HcResult<()> {
        let blob = self
            .secrets
            .get(id_str)
            .ok_or(HolochainError::new("Secret not found"))?;
        let mut passphrase = self.passphrase_manager.as_ref()?.get_passphrase()?;
        let secret = match blob.blob_type {
            BlobType::Seed => Secret::Seed(Seed::from_blob(blob, &mut passphrase, self.hash_config.clone())?.buf),
            BlobType::SigningKey => {
                Secret::SigningKey(SigningKeyPair::from_blob(blob, &mut passphrase, self.hash_config.clone())?)
            }
            BlobType::EncryptingKey => {
                Secret::EncryptingKey(EncryptingKeyPair::from_blob(blob, &mut passphrase, self.hash_config.clone())?)
            }
            _ => {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Tried to decrypt unsupported BlobType in Keystore: {}",
                    id_str
                )));
            }
        };
        self.cache
            .insert(id_str.clone(), Arc::new(Mutex::new(secret)));
        Ok(())
    }

    fn encrypt(&mut self, id_str: &String) -> HcResult<()> {
        let secret = self
            .cache
            .get(id_str)
            .ok_or(HolochainError::new("Secret not found"))?;
        let mut passphrase = self.passphrase_manager.as_ref()?.get_passphrase()?;
        self.check_passphrase(&mut passphrase)?;
        let blob = match *secret.lock()? {
            Secret::Seed(ref mut buf) => {
                let mut owned_buf = SecBuf::with_insecure(buf.len());
                owned_buf.write(0, &*buf.read_lock())?;
                Seed::new(owned_buf, SeedType::OneShot).as_blob(
                    &mut passphrase,
                    "".to_string(),
                    self.hash_config.clone(),
                )
            }
            Secret::SigningKey(ref mut key) => key.as_blob(&mut passphrase, "".to_string(), self.hash_config.clone()),
            Secret::EncryptingKey(ref mut key) => {
                key.as_blob(&mut passphrase, "".to_string(), self.hash_config.clone())
            }
        }?;
        self.secrets.insert(id_str.clone(), blob);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save(&self, path: PathBuf) -> HcResult<()> {
        let json_string = serde_json::to_string(self)?;
        let mut file = File::create(path)?;
        file.write_all(&json_string.as_bytes())?;
        Ok(())
    }

    /// return a list of the identifiers stored in the keystore
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<String> {
        self.secrets.keys().map(|k| k.to_string()).collect()
    }

    /// adds a secret to the keystore
    #[allow(dead_code)]
    pub fn add(&mut self, dst_id_str: &str, secret: Arc<Mutex<Secret>>) -> HcResult<()> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        self.cache.insert(dst_id.clone(), secret);
        self.encrypt(&dst_id)?;
        Ok(())
    }

    /// adds a random root seed into the keystore
    #[allow(dead_code)]
    pub fn add_random_seed(&mut self, dst_id_str: &str, size: usize) -> HcResult<()> {
        let dst_id = self.check_dst_identifier(dst_id_str)?;
        let seed_buf = generate_random_buf(size);
        let secret = Arc::new(Mutex::new(Secret::Seed(seed_buf)));
        self.cache.insert(dst_id.clone(), secret);
        self.encrypt(&dst_id)?;
        Ok(())
    }

    fn check_dst_identifier(&self, dst_id_str: &str) -> HcResult<String> {
        let dst_id = dst_id_str.to_string();
        if self.secrets.contains_key(&dst_id) {
            return Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string(),
            ));
        }
        Ok(dst_id)
    }

    /// gets a secret from the keystore
    #[allow(dead_code)]
    pub fn get(&mut self, src_id_str: &str) -> HcResult<Arc<Mutex<Secret>>> {
        let src_id = src_id_str.to_string();
        if !self.secrets.contains_key(&src_id) {
            return Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string(),
            ));
        }

        if !self.cache.contains_key(&src_id) {
            self.decrypt(&src_id)?;
        }

        Ok(self.cache.get(&src_id).unwrap().clone()) // unwrap ok because we made sure src exists
    }

    fn check_identifiers(
        &mut self,
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
        self.cache.insert(dst_id.clone(), secret);
        self.encrypt(&dst_id)?;

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
        self.cache.insert(dst_id.clone(), secret);
        self.encrypt(&dst_id)?;

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

    /// adds an encrypting keypair into the keystore based on a seed already in the keystore
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

    /// adds a keybundle into the keystore based on a seed already in the keystore by
    /// adding two keypair secrets (signing and encrypting) under the named prefix
    /// returns the public keys of the secrets
    #[allow(dead_code)]
    pub fn add_keybundle_from_seed(
        &mut self,
        src_id_str: &str,
        dst_id_prefix_str: &str,
        context: &SeedContext,
        index: u64,
    ) -> HcResult<(Base32, Base32)> {
        let dst_sign_id_str = [dst_id_prefix_str, KEYBUNDLE_SIGNKEY_SUFFIX].join("");
        let dst_enc_id_str = [dst_id_prefix_str, KEYBUNDLE_ENCKEY_SUFFIX].join("");

        let sign_pub_key = self.add_key_from_seed(
            src_id_str,
            &dst_sign_id_str,
            context,
            index,
            KeyType::Signing,
        )?;
        let enc_pub_key = self.add_key_from_seed(
            src_id_str,
            &dst_enc_id_str,
            context,
            index,
            KeyType::Encrypting,
        )?;
        Ok((sign_pub_key, enc_pub_key))
    }

    /// adds a keybundle into the keystore based on a seed already in the keystore by
    /// adding two keypair secrets (signing and encrypting) under the named prefix
    /// returns the public keys of the secrets
    #[allow(dead_code)]
    pub fn get_keybundle(&mut self, src_id_prefix_str: &str) -> HcResult<KeyBundle> {
        let src_sign_id_str = [src_id_prefix_str, KEYBUNDLE_SIGNKEY_SUFFIX].join("");
        let src_enc_id_str = [src_id_prefix_str, KEYBUNDLE_ENCKEY_SUFFIX].join("");

        let sign_secret = self.get(&src_sign_id_str)?;
        let mut sign_secret = sign_secret.lock().unwrap();
        let sign_key = match *sign_secret {
            Secret::SigningKey(ref mut key_pair) => {
                let mut buf = SecBuf::with_secure(key_pair.private().len());
                let pub_key = key_pair.public();
                let lock = key_pair.private().read_lock();
                buf.write(0, &**lock)?;
                SigningKeyPair::new(pub_key, buf)
            }
            _ => {
                return Err(HolochainError::ErrorGeneric(
                    "source secret is not a signing key".to_string(),
                ));
            }
        };

        let enc_secret = self.get(&src_enc_id_str)?;
        let mut enc_secret = enc_secret.lock().unwrap();
        let enc_key = match *enc_secret {
            Secret::EncryptingKey(ref mut key_pair) => {
                let mut buf = SecBuf::with_secure(key_pair.private().len());
                let pub_key = key_pair.public();
                let lock = key_pair.private().read_lock();
                buf.write(0, &**lock)?;
                EncryptingKeyPair::new(pub_key, buf)
            }
            _ => {
                return Err(HolochainError::ErrorGeneric(
                    "source secret is not an encrypting key".to_string(),
                ));
            }
        };

        Ok(KeyBundle::new(sign_key, enc_key)?)
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

pub fn test_hash_config() -> Option<PwHashConfig> {
    Some(PwHashConfig(
        OPSLIMIT_INTERACTIVE,
        MEMLIMIT_INTERACTIVE,
        ALG_ARGON2ID13,
    ))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use base64;
    use conductor::passphrase_manager::PassphraseServiceMock;
    use holochain_dpki::{utils, AGENT_ID_CTX};

    fn mock_passphrase_manager(passphrase: String) -> Arc<PassphraseManager> {
        Arc::new(PassphraseManager::new(Arc::new(Mutex::new(
            PassphraseServiceMock { passphrase },
        ))))
    }

    fn new_test_keystore(passphrase: String) -> Keystore {
        Keystore::new(mock_passphrase_manager(passphrase), test_hash_config()).unwrap()
    }

    fn random_test_passphrase() -> String {
        let mut buf = utils::generate_random_buf(10);
        let read_lock = buf.read_lock();
        String::from_utf8_lossy(&*read_lock).to_string()
    }

    #[test]
    fn test_keystore_new() {
        let random_passphrase = random_test_passphrase();
        let keystore = new_test_keystore(random_passphrase.clone());
        let mut random_passphrase = SecBuf::with_insecure_from_string(random_passphrase);
        assert!(keystore.list().is_empty());
        assert_eq!(keystore.check_passphrase(&mut random_passphrase), Ok(true));
        let mut another_random_passphrase = utils::generate_random_buf(10);
        assert_eq!(
            keystore.check_passphrase(&mut another_random_passphrase),
            Ok(false)
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        let random_passphrase = random_test_passphrase();
        let mut keystore = new_test_keystore(random_passphrase.clone());
        assert_eq!(keystore.add_random_seed("my_root_seed", SEED_SIZE), Ok(()));
        assert_eq!(keystore.list(), vec!["my_root_seed".to_string()]);

        let mut path = PathBuf::new();
        path.push("tmp-test/test-keystore");
        keystore.save(path.clone()).unwrap();

        let mut loaded_keystore =
            Keystore::new_from_file(path.clone(), mock_passphrase_manager(random_passphrase), test_hash_config())
                .unwrap();
        assert_eq!(loaded_keystore.list(), vec!["my_root_seed".to_string()]);

        let secret1 = keystore.get("my_root_seed").unwrap();
        let expected_seed = match *secret1.lock().unwrap() {
            Secret::Seed(ref mut buf) => {
                let lock = buf.read_lock();
                String::from_utf8_lossy(&**lock).to_string()
            }
            _ => unreachable!(),
        };

        let secret2 = loaded_keystore.get("my_root_seed").unwrap();
        let loaded_seed = match *secret2.lock().unwrap() {
            Secret::Seed(ref mut buf) => {
                let lock = buf.read_lock();
                String::from_utf8_lossy(&**lock).to_string()
            }
            _ => unreachable!(),
        };

        assert_eq!(expected_seed, loaded_seed);
    }

    #[test]
    fn test_keystore_change_passphrase() {
        let random_passphrase = random_test_passphrase();
        let mut keystore = new_test_keystore(random_passphrase.clone());
        let mut random_passphrase = SecBuf::with_insecure_from_string(random_passphrase);
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
        let mut keystore = new_test_keystore(random_test_passphrase());

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
        let mut keystore = new_test_keystore(random_test_passphrase());

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
        let mut keystore = new_test_keystore(random_test_passphrase());
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
        let mut keystore = new_test_keystore(random_test_passphrase());
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

    #[test]
    fn test_keystore_keybundle() {
        let mut keystore = new_test_keystore(random_test_passphrase());
        let context = SeedContext::new(AGENT_ID_CTX);

        assert_eq!(
            keystore.add_keybundle_from_seed("my_root_seed", "my_keybundle", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "unknown source identifier".to_string()
            ))
        );

        let _ = keystore.add_random_seed("my_root_seed", SEED_SIZE);

        let result = keystore.add_keybundle_from_seed("my_root_seed", "my_keybundle", &context, 1);
        assert!(!result.is_err());
        let (sign_pubkey, enc_pubkey) = result.unwrap();
        assert!(format!("{}", sign_pubkey).starts_with("Hc"));

        assert_eq!(
            keystore.add_keybundle_from_seed("my_root_seed", "my_keybundle", &context, 1),
            Err(HolochainError::ErrorGeneric(
                "identifier already exists".to_string()
            ))
        );

        let result = keystore.get_keybundle("my_keybundle");
        assert!(!result.is_err());
        let key_bundle = result.unwrap();

        assert_eq!(key_bundle.sign_keys.public(), sign_pubkey);
        assert_eq!(key_bundle.enc_keys.public(), enc_pubkey);
    }

}
