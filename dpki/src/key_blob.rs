#![allow(warnings)]
use holochain_sodium::{kx, secbuf::SecBuf, sign, *};

use crate::{
    key_bundle::*,
    keypair::*,
    password_encryption::{self, pw_dec, pw_enc, pw_hash, EncryptedData, PwHashConfig},
    seed::*,
    utils, SEED_SIZE,
};
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
};
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
    // TODO futur blobbables?
    // KeyPair,
    // Key,
}

/// Trait to implement in order to be blobbable into a KeyBlob
pub trait Blobbable {
    fn blob_type() -> BlobType;
    fn blob_size() -> usize;

    fn from_blob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<Self>
    where
        Self: Sized;

    fn as_blob(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBlob>;

    // -- Common methods -- //

    /// Blobs a data buf
    fn finalize_blobbing(
        data_buf: &mut SecBuf,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<String> {
        // Check size
        if data_buf.len() != Self::blob_size() {
            return Err(HolochainError::ErrorGeneric(
                "Invalid buf size for Blobbing".to_string(),
            ));
        }

        utils::encrypt_with_passphrase_buf(data_buf, passphrase, config)
    }

    /// Get the data buf back from a Blob
    fn unblob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<SecBuf> {
        // Check type
        if blob.blob_type != Self::blob_type() {
            return Err(HolochainError::ErrorGeneric(
                "Blob type mismatch while unblobbing".to_string(),
            ));
        }
        utils::decrypt_with_passphrase_buf(&blob.data, passphrase, config, Self::blob_size())
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
    /// @param {Option<PwHashConfig>} config - Settings for pwhash
    /// @return Resulting Seed
    fn from_blob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<Self> {
        // Retrieve data buf from blob
        let mut seed_buf = Self::unblob(blob, passphrase, config)?;
        // Construct
        Ok(Seed::new(seed_buf, blob.seed_type.clone()))
    }

    ///  generate a persistence bundle with hint info
    ///  @param {string} passphrase - the encryption passphrase
    ///  @param {string} hint - additional info / description for persistence
    /// @param {Option<PwHashConfig>} config - Settings for pwhash
    /// @return {KeyBlob} - bundle of the seed
    fn as_blob(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBlob> {
        // Blob seed buf directly
        let encoded_blob = Self::finalize_blobbing(&mut self.buf, passphrase, config)?;
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

const KEYBUNDLE_BLOB_SIZE: usize = 1 // version byte
    + sign::PUBLICKEYBYTES
    + kx::PUBLICKEYBYTES
    + sign::SECRETKEYBYTES
    + kx::SECRETKEYBYTES;

pub const KEYBUNDLE_BLOB_SIZE_ALIGNED: usize = ((KEYBUNDLE_BLOB_SIZE + 8 - 1) / 8) * 8;

impl Blobbable for KeyBundle {
    fn blob_type() -> BlobType {
        BlobType::KeyBundle
    }

    fn blob_size() -> usize {
        KEYBUNDLE_BLOB_SIZE_ALIGNED
    }

    /// Generate an encrypted blob for persistence
    /// @param {SecBuf} passphrase - the encryption passphrase
    /// @param {string} hint - additional info / description for the bundle
    /// @param {Option<PwHashConfig>} config - Settings for pwhash
    fn as_blob(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBlob> {
        // Initialize buffer
        let mut data_buf = SecBuf::with_secure(KEYBUNDLE_BLOB_SIZE_ALIGNED);
        let mut offset: usize = 0;
        // Write version
        data_buf.write(0, &[KEYBUNDLE_BLOB_FORMAT_VERSION]).unwrap();
        offset += 1;
        // Write public signing key
        let key = self.sign_keys.decode_pub_key();
        assert_eq!(sign::PUBLICKEYBYTES, key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public signing key");
        offset += sign::PUBLICKEYBYTES;
        // Write public encoding key
        let key = self.enc_keys.decode_pub_key();
        assert_eq!(kx::PUBLICKEYBYTES, key.len());
        data_buf
            .write(offset, &key)
            .expect("Failed blobbing public encoding key");
        offset += kx::PUBLICKEYBYTES;
        // Write private signing key
        data_buf
            .write(offset, &**self.sign_keys.private.read_lock())
            .expect("Failed blobbing private signing key");
        offset += sign::SECRETKEYBYTES;
        // Write private encoding key
        data_buf
            .write(offset, &**self.enc_keys.private.read_lock())
            .expect("Failed blobbing private encoding key");
        offset += kx::SECRETKEYBYTES;
        assert_eq!(offset, KEYBUNDLE_BLOB_SIZE);

        // Finalize
        let encoded_blob = Self::finalize_blobbing(&mut data_buf, passphrase, config)?;

        // Done
        Ok(KeyBlob {
            seed_type: self.seed_type.clone(),
            blob_type: BlobType::KeyBundle,
            hint,
            data: encoded_blob,
        })
    }

    /// Construct the pairs from an encrypted blob
    /// @param {object} bundle - persistence info
    /// @param {SecBuf} passphrase - decryption passphrase
    /// @param {Option<PwHashConfig>} config - Settings for pwhash
    fn from_blob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBundle> {
        // Retrieve data buf from blob
        let mut keybundle_blob = Self::unblob(blob, passphrase, config)?;

        // Deserialize manually
        let mut pub_sign = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut pub_enc = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
        let mut priv_sign = SecBuf::with_secure(sign::SECRETKEYBYTES);
        let mut priv_enc = SecBuf::with_secure(kx::SECRETKEYBYTES);
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
            seed_type: blob.seed_type.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{key_bundle::tests::*, utils::generate_random_seed_buf, SEED_SIZE};
    use holochain_sodium::pwhash;

    #[test]
    fn it_should_blob_keybundle() {
        let mut seed_buf = generate_random_seed_buf(SEED_SIZE);
        let mut passphrase = generate_random_seed_buf(SEED_SIZE);

        let mut bundle = KeyBundle::new_from_seed_buf(&mut seed_buf, SeedType::Mock).unwrap();

        let blob = bundle
            .as_blob(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        println!("blob.data: {}", blob.data);

        assert_eq!(SeedType::Mock, blob.seed_type);
        assert_eq!("hint", blob.hint);

        let mut unblob = KeyBundle::from_blob(&blob, &mut passphrase, TEST_CONFIG).unwrap();

        assert!(bundle.is_same(&mut unblob));

        // Test with wrong passphrase
        passphrase.randomize();
        let maybe_unblob = KeyBundle::from_blob(&blob, &mut passphrase, TEST_CONFIG);
        assert!(maybe_unblob.is_err());
    }

    #[test]
    fn it_should_blob_seed() {
        let mut passphrase = generate_random_seed_buf(SEED_SIZE);
        let mut seed_buf = generate_random_seed_buf(SEED_SIZE);
        let mut initial_seed = Seed::new(seed_buf, SeedType::Root);

        let blob = initial_seed
            .as_blob(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let mut root_seed = Seed::from_blob(&blob, &mut passphrase, TEST_CONFIG).unwrap();
        assert_eq!(SeedType::Root, root_seed.kind);
        assert_eq!(0, root_seed.buf.compare(&mut initial_seed.buf));
    }

    #[test]
    fn it_should_blob_device_pin_seed() {
        let mut passphrase = generate_random_seed_buf(SEED_SIZE);
        let mut seed_buf = generate_random_seed_buf(SEED_SIZE);
        let mut initial_device_pin_seed = DevicePinSeed::new(seed_buf);

        let blob = initial_device_pin_seed
            .seed_mut()
            .as_blob(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let seed = Seed::from_blob(&blob, &mut passphrase, TEST_CONFIG).unwrap();
        let mut typed_seed = seed.into_typed().unwrap();

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
