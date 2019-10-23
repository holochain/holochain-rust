use crate::{key_bundle::*, keypair::*, seed::*, utils, SecBuf, CRYPTO, SEED_SIZE};
use holochain_core_types::error::{HcResult, HolochainError};
use std::str;

use serde_derive::{Deserialize, Serialize};

/// The data includes a base64 encoded, json serialized string of the EncryptedData that
/// was created by concatenating all the keys in one SecBuf
#[derive(Serialize, Deserialize)]
pub struct KeyBlob {
    pub blob_type: BlobType,
    pub seed_type: SeedType,
    pub hint: String,
    ///  base64 encoded, json serialized string of the EncryptedData
    pub data: String,
}

/// Enum of all blobbable types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BlobType {
    Seed,
    KeyBundle,
    SigningKey,
    EncryptingKey,
    // TODO futur blobbables?
    // Key,
}

/// Trait to implement in order to be blobbable into a KeyBlob
pub trait Blobbable {
    fn blob_type() -> BlobType;
    fn blob_size() -> usize;

    fn from_blob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<Self>
    where
        Self: Sized;

    fn as_blob(&mut self, passphrase: &mut SecBuf, hint: String) -> HcResult<KeyBlob>;

    // -- Common methods -- //

    /// Blobs a data buf
    fn finalize_blobbing(data_buf: &mut SecBuf, passphrase: &mut SecBuf) -> HcResult<String> {
        // Check size
        if data_buf.len() != Self::blob_size() {
            return Err(HolochainError::ErrorGeneric(
                "Invalid buf size for Blobbing".to_string(),
            ));
        }

        utils::encrypt_with_passphrase_buf(data_buf, passphrase)
    }

    /// Get the data buf back from a Blob
    fn unblob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<SecBuf> {
        // Check type
        if blob.blob_type != Self::blob_type() {
            return Err(HolochainError::ErrorGeneric(
                "Blob type mismatch while unblobbing".to_string(),
            ));
        }
        utils::decrypt_with_passphrase_buf(&blob.data, passphrase, Self::blob_size())
    }
}

//--------------------------------------------------------------------------------------------------
// Seed
//--------------------------------------------------------------------------------------------------

impl Blobbable for Seed {
    fn blob_type() -> BlobType {
        BlobType::Seed
    }

    fn blob_size() -> usize {
        SEED_SIZE
    }

    /// Get the Seed from a Seed Blob
    /// @param {object} blob - the seed blob to unblob
    /// @param {string} passphrase - the decryption passphrase
    /// @return Resulting Seed
    fn from_blob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<Self> {
        // Retrieve data buf from blob
        let seed_buf = Self::unblob(blob, passphrase)?;
        // Construct
        Ok(Seed::new(seed_buf, blob.seed_type.clone()))
    }

    ///  generate a persistence bundle with hint info
    ///  @param {string} passphrase - the encryption passphrase
    ///  @param {string} hint - additional info / description for persistence
    /// @return {KeyBlob} - bundle of the seed
    fn as_blob(&mut self, passphrase: &mut SecBuf, hint: String) -> HcResult<KeyBlob> {
        // Blob seed buf directly
        let encoded_blob = Self::finalize_blobbing(&mut self.buf, passphrase)?;
        // Done
        Ok(KeyBlob {
            seed_type: self.kind.clone(),
            blob_type: BlobType::Seed,
            hint,
            data: encoded_blob,
        })
    }
}

//--------------------------------------------------------------------------------------------------
// KeyBundle
//--------------------------------------------------------------------------------------------------

const KEYBUNDLE_BLOB_FORMAT_VERSION: u8 = 2;

lazy_static! {
    pub static ref KEYBUNDLE_BLOB_SIZE: usize =
        1 // version byte
        + CRYPTO.sign_secret_key_bytes()
        + CRYPTO.sign_public_key_bytes()
        + CRYPTO.sign_secret_key_bytes()
        + CRYPTO.sign_secret_key_bytes();
    pub static ref KEYBUNDLE_BLOB_SIZE_ALIGNED: usize = ((*KEYBUNDLE_BLOB_SIZE + 8 - 1) / 8) * 8;
}

impl Blobbable for KeyBundle {
    fn blob_type() -> BlobType {
        BlobType::KeyBundle
    }

    fn blob_size() -> usize {
        *KEYBUNDLE_BLOB_SIZE_ALIGNED
    }

    /// Generate an encrypted blob for persistence
    /// @param {SecBuf} passphrase - the encryption passphrase
    /// @param {string} hint - additional info / description for the bundle
    fn as_blob(&mut self, passphrase: &mut SecBuf, hint: String) -> HcResult<KeyBlob> {
        // Initialize buffer
        let mut data_buf = CRYPTO.buf_new_secure(KeyBundle::blob_size());
        let mut offset: usize = 0;
        // Write version
        data_buf.write(0, &[KEYBUNDLE_BLOB_FORMAT_VERSION]).unwrap();
        offset += 1;
        // Write public signing key
        let key = self.sign_keys.decode_pub_key();
        assert_eq!(CRYPTO.sign_secret_key_bytes(), key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public signing key");
        offset += CRYPTO.sign_secret_key_bytes();
        // Write public encoding key
        let key = self.enc_keys.decode_pub_key();
        assert_eq!(CRYPTO.sign_public_key_bytes(), key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public encoding key");
        offset += CRYPTO.sign_public_key_bytes();
        // Write private signing key
        data_buf
            .write(offset, &*self.sign_keys.private.read_lock())
            .expect("Failed blobbing private signing key");
        offset += CRYPTO.sign_secret_key_bytes();
        // Write private encoding key
        data_buf
            .write(offset, &*self.enc_keys.private.read_lock())
            .expect("Failed blobbing private encoding key");
        offset += CRYPTO.sign_secret_key_bytes();
        assert_eq!(offset, *KEYBUNDLE_BLOB_SIZE);

        // Finalize
        let encoded_blob = Self::finalize_blobbing(&mut data_buf, passphrase)?;

        // Done
        Ok(KeyBlob {
            seed_type: SeedType::Mock,
            blob_type: BlobType::KeyBundle,
            hint,
            data: encoded_blob,
        })
    }

    /// Construct the pairs from an encrypted blob
    /// @param {object} bundle - persistence info
    /// @param {SecBuf} passphrase - decryption passphrase
    fn from_blob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<KeyBundle> {
        // Retrieve data buf from blob
        let keybundle_blob = Self::unblob(blob, passphrase)?;

        // Deserialize manually
        let mut pub_sign = CRYPTO.buf_new_insecure(CRYPTO.sign_secret_key_bytes());
        let mut pub_enc = CRYPTO.buf_new_insecure(CRYPTO.sign_public_key_bytes());
        let mut priv_sign = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        let mut priv_enc = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        {
            let keybundle_blob = keybundle_blob.read_lock();
            if keybundle_blob[0] != KEYBUNDLE_BLOB_FORMAT_VERSION {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Invalid KeyBundle Blob Format: v{:?} != v{:?}",
                    keybundle_blob[0], KEYBUNDLE_BLOB_FORMAT_VERSION
                )));
            }
            pub_sign.write(0, &keybundle_blob[1..33])?;
            pub_enc.write(0, &keybundle_blob[33..65])?;
            priv_sign.write(0, &keybundle_blob[65..129])?;
            priv_enc.write(0, &keybundle_blob[129..161])?;
        }
        // Done
        Ok(KeyBundle {
            sign_keys: SigningKeyPair::new(
                SigningKeyPair::encode_pub_key(&mut pub_sign),
                priv_sign,
            ),
            enc_keys: EncryptingKeyPair::new(
                EncryptingKeyPair::encode_pub_key(&mut pub_enc),
                priv_enc,
            ),
        })
    }
}

//--------------------------------------------------------------------------------------------------
// SigningKey
//--------------------------------------------------------------------------------------------------

const SIGNING_KEY_BLOB_FORMAT_VERSION: u8 = 1;

lazy_static! {
    pub static ref SIGNING_KEY_BLOB_SIZE: usize = 1 // version byte
        + CRYPTO.sign_secret_key_bytes()
        + CRYPTO.sign_secret_key_bytes();
    pub static ref SIGNING_KEY_BLOB_SIZE_ALIGNED: usize = ((*SIGNING_KEY_BLOB_SIZE + 8 - 1) / 8) * 8;
}

impl Blobbable for SigningKeyPair {
    fn blob_type() -> BlobType {
        BlobType::SigningKey
    }

    fn blob_size() -> usize {
        *SIGNING_KEY_BLOB_SIZE_ALIGNED
    }

    /// Generate an encrypted blob for persistence
    /// @param {SecBuf} passphrase - the encryption passphrase
    /// @param {string} hint - additional info / description for the bundle
    fn as_blob(&mut self, passphrase: &mut SecBuf, hint: String) -> HcResult<KeyBlob> {
        // Initialize buffer
        let mut data_buf = CRYPTO.buf_new_secure(SigningKeyPair::blob_size());
        let mut offset: usize = 0;
        // Write version
        data_buf
            .write(0, &[SIGNING_KEY_BLOB_FORMAT_VERSION])
            .unwrap();
        offset += 1;
        // Write public signing key
        let key = self.decode_pub_key();
        assert_eq!(CRYPTO.sign_secret_key_bytes(), key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public signing key");
        offset += CRYPTO.sign_secret_key_bytes();
        // Write private signing key
        data_buf
            .write(offset, &*self.private.read_lock())
            .expect("Failed blobbing private signing key");
//        offset += CRYPTO.sign_secret_key_bytes();

        // Finalize
        let encoded_blob = Self::finalize_blobbing(&mut data_buf, passphrase)?;

        // Done
        Ok(KeyBlob {
            seed_type: SeedType::Mock,
            blob_type: BlobType::SigningKey,
            hint,
            data: encoded_blob,
        })
    }

    /// Construct the pairs from an encrypted blob
    /// @param {object} bundle - persistence info
    /// @param {SecBuf} passphrase - decryption passphrase
    fn from_blob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<SigningKeyPair> {
        // Retrieve data buf from blob
        let keybundle_blob = Self::unblob(blob, passphrase)?;

        // Deserialize manually
        let mut pub_sign = CRYPTO.buf_new_insecure(CRYPTO.sign_secret_key_bytes());
        let mut priv_sign = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        {
            let keybundle_blob = keybundle_blob.read_lock();
            if keybundle_blob[0] != SIGNING_KEY_BLOB_FORMAT_VERSION {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Invalid SigningKey Blob Format: v{:?} != v{:?}",
                    keybundle_blob[0], SIGNING_KEY_BLOB_FORMAT_VERSION
                )));
            }
            pub_sign.write(0, &keybundle_blob[1..33])?;
            priv_sign.write(0, &keybundle_blob[33..97])?;
        }
        // Done
        Ok(SigningKeyPair::new(
            SigningKeyPair::encode_pub_key(&mut pub_sign),
            priv_sign,
        ))
    }
}

//--------------------------------------------------------------------------------------------------
// EncryptingKey
//--------------------------------------------------------------------------------------------------

const ENCRYPTING_KEY_BLOB_FORMAT_VERSION: u8 = 1;

lazy_static! {
    pub static ref ENCRYPTING_KEY_BLOB_SIZE: usize = 1 // version byte
        + CRYPTO.sign_public_key_bytes()
        + CRYPTO.sign_secret_key_bytes();

    pub static ref ENCRYPTING_KEY_BLOB_SIZE_ALIGNED: usize = ((*ENCRYPTING_KEY_BLOB_SIZE + 8 - 1) / 8) * 8;
}

impl Blobbable for EncryptingKeyPair {
    fn blob_type() -> BlobType {
        BlobType::EncryptingKey
    }

    fn blob_size() -> usize {
        *ENCRYPTING_KEY_BLOB_SIZE_ALIGNED
    }

    /// Generate an encrypted blob for persistence
    /// @param {SecBuf} passphrase - the encryption passphrase
    /// @param {string} hint - additional info / description for the bundle
    fn as_blob(&mut self, passphrase: &mut SecBuf, hint: String) -> HcResult<KeyBlob> {
        // Initialize buffer
        let mut data_buf = CRYPTO.buf_new_secure(EncryptingKeyPair::blob_size());
        let mut offset: usize = 0;
        // Write version
        data_buf
            .write(0, &[ENCRYPTING_KEY_BLOB_FORMAT_VERSION])
            .unwrap();
        offset += 1;
        // Write public encrypting key
        let key = self.decode_pub_key();
        assert_eq!(CRYPTO.sign_public_key_bytes(), key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public encrypting key");
        offset += CRYPTO.sign_public_key_bytes();
        // Write private encyrpting key
        data_buf
            .write(offset, &*self.private.read_lock())
            .expect("Failed blobbing private ecrypting key");
//        offset += CRYPTO.sign_secret_key_bytes();

        // Finalize
        let encoded_blob = Self::finalize_blobbing(&mut data_buf, passphrase)?;

        // Done
        Ok(KeyBlob {
            seed_type: SeedType::Mock,
            blob_type: BlobType::EncryptingKey,
            hint,
            data: encoded_blob,
        })
    }

    /// Construct the pairs from an encrypted blob
    /// @param {object} bundle - persistence info
    /// @param {SecBuf} passphrase - decryption passphrase
    fn from_blob(blob: &KeyBlob, passphrase: &mut SecBuf) -> HcResult<EncryptingKeyPair> {
        // Retrieve data buf from blob
        let keybundle_blob = Self::unblob(blob, passphrase)?;

        // Deserialize manually
        let mut pub_sign = CRYPTO.buf_new_insecure(CRYPTO.sign_public_key_bytes());
        let mut priv_sign = CRYPTO.buf_new_secure(CRYPTO.sign_secret_key_bytes());
        {
            let keybundle_blob = keybundle_blob.read_lock();
            if keybundle_blob[0] != ENCRYPTING_KEY_BLOB_FORMAT_VERSION {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Invalid EncryptingKey Blob Format: v{:?} != v{:?}",
                    keybundle_blob[0], ENCRYPTING_KEY_BLOB_FORMAT_VERSION
                )));
            }
            pub_sign.write(0, &keybundle_blob[1..33])?;
            priv_sign.write(0, &keybundle_blob[33..65])?;
        }
        // Done
        Ok(EncryptingKeyPair::new(
            EncryptingKeyPair::encode_pub_key(&mut pub_sign),
            priv_sign,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        keypair::{generate_random_enc_keypair, generate_random_sign_keypair},
        utils::{generate_random_seed_buf, tests::TEST_CRYPTO},
    };

    #[test]
    fn it_should_blob_keybundle() {
        let mut seed_buf = generate_random_seed_buf();
        let mut passphrase = generate_random_seed_buf();

        let mut bundle = KeyBundle::new_from_seed_buf(&mut seed_buf).unwrap();

        let blob = bundle.as_blob(&mut passphrase, "hint".to_string()).unwrap();

        println!("blob.data: {}", blob.data);

        assert_eq!(SeedType::Mock, blob.seed_type);
        assert_eq!("hint", blob.hint);

        let mut unblob = KeyBundle::from_blob(&blob, &mut passphrase).unwrap();

        assert!(bundle.is_same(&mut unblob));

        // Test with wrong passphrase
        TEST_CRYPTO.randombytes_buf(&mut passphrase).expect("should work");
        let maybe_unblob = KeyBundle::from_blob(&blob, &mut passphrase);
        assert!(maybe_unblob.is_err());
    }

    #[test]
    fn it_should_blob_signing_key() {
        let mut passphrase = generate_random_seed_buf();

        let mut signing_key = generate_random_sign_keypair().unwrap();

        let blob = signing_key
            .as_blob(&mut passphrase, "hint".to_string())
            .unwrap();

        println!("blob.data: {}", blob.data);

        assert_eq!(SeedType::Mock, blob.seed_type);
        assert_eq!("hint", blob.hint);

        let mut unblob = SigningKeyPair::from_blob(&blob, &mut passphrase).unwrap();

        assert_eq!(0, unblob.private().compare(&mut signing_key.private()));
        assert_eq!(unblob.public(), signing_key.public());

        // Test with wrong passphrase
        TEST_CRYPTO.randombytes_buf(&mut passphrase).expect("should work");
        let maybe_unblob = SigningKeyPair::from_blob(&blob, &mut passphrase);
        assert!(maybe_unblob.is_err());
    }

    #[test]
    fn it_should_blob_encrypting_key() {
        let mut passphrase = generate_random_seed_buf();

        let mut enc_key = generate_random_enc_keypair().unwrap();

        let blob = enc_key
            .as_blob(&mut passphrase, "hint".to_string())
            .unwrap();

        println!("blob.data: {}", blob.data);

        assert_eq!(SeedType::Mock, blob.seed_type);
        assert_eq!("hint", blob.hint);

        let mut unblob = EncryptingKeyPair::from_blob(&blob, &mut passphrase).unwrap();

        assert_eq!(0, unblob.private().compare(&mut enc_key.private()));
        assert_eq!(unblob.public(), enc_key.public());

        // Test with wrong passphrase
        TEST_CRYPTO.randombytes_buf(&mut passphrase).expect("should work");
        let maybe_unblob = EncryptingKeyPair::from_blob(&blob, &mut passphrase);
        assert!(maybe_unblob.is_err());
    }

    #[test]
    fn it_should_blob_seed() {
        let mut passphrase = generate_random_seed_buf();
        let seed_buf = generate_random_seed_buf();
        let mut initial_seed = Seed::new(seed_buf, SeedType::Root);

        let blob = initial_seed
            .as_blob(&mut passphrase, "hint".to_string())
            .unwrap();

        let root_seed = Seed::from_blob(&blob, &mut passphrase).unwrap();
        assert_eq!(SeedType::Root, root_seed.kind);
        assert_eq!(0, root_seed.buf.compare(&mut initial_seed.buf));
    }

    #[test]
    fn it_should_blob_device_pin_seed() {
        let mut passphrase = generate_random_seed_buf();
        let seed_buf = generate_random_seed_buf();
        let mut initial_device_pin_seed = DevicePinSeed::new(seed_buf);

        let blob = initial_device_pin_seed
            .seed_mut()
            .as_blob(&mut passphrase, "hint".to_string())
            .unwrap();

        let seed = Seed::from_blob(&blob, &mut passphrase).unwrap();
        let typed_seed = seed.into_typed().unwrap();

        match typed_seed {
            TypedSeed::DevicePin(mut device_pin_seed) => {
                assert_eq!(
                    0,
                    device_pin_seed
                        .seed_mut()
                        .buf
                        .compare(&mut initial_device_pin_seed.seed_mut().buf)
                );
            }
            _ => unreachable!(),
        }
    }
}
