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

pub enum BlobType {
    Seed,
    KeyBundle,
    KeyPair,
    Key,
}

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

pub trait Blobbable {
    fn blob_type() -> BlobType;
    fn blob_size() -> usize;

    fn from_blob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<Self>;

    fn as_blob(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBlob>;

    // -- Private methods -- //

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
        // encrypt buffer
        let encrypted_blob = password_encryption::pw_enc(data_buf, passphrase, config)?;
        // Serialize and convert to base64
        let serialized_blob =
            serde_json::to_string(&encrypted_blob).expect("Failed to serialize Blob");
        Ok(base64::encode(&serialized_blob)?)
    }

    //
    fn unblob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<Self> {
        // Check type
        if blob.blob_type != Self::blob_type() {
            return Err(HolochainError::ErrorGeneric(
                "Blob type mismatch while unblobbing".to_string(),
            ));
        }
        // Decode base64
        let blob_b64 = base64::decode(&blob.data)?;
        // Deserialize
        let blob_json = str::from_utf8(&blob_b64)?;
        let encrypted_blob: EncryptedData = serde_json::from_str(&blob_json)?;
        // Decrypt
        let mut decrypted_data = SecBuf::with_secure(Self::blob_size());
        pw_dec(&encrypted_blob, passphrase, &mut decrypted_data, config)?;
        // Check size
        if decrypted_data.len() != Self::blob_size() {
            return Err(HolochainError::ErrorGeneric(
                "Invalid Blob size".to_string(),
            ));
        }
        // Done
        Ok(decrypted_data)
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
    ) -> HcResult<TypedSeed> {
        // Retrieve data buf from blob
        let mut seed_buf = Self::unblob(blob, passphrase, config)?;
        // Construct
        match blob.seed_type {
            SeedType::Root => Ok(TypedSeed::Root(RootSeed::new(seed_buf))),
            SeedType::Device => Ok(TypedSeed::Device(DeviceSeed::new(seed_buf))),
            SeedType::DevicePin => Ok(TypedSeed::DevicePin(DevicePinSeed::new(seed_buf))),
            _ => Err(HolochainError::new(&format!(
                "Unblobbing seed of type '{:?}' not allowed",
                blob.seed_type
            ))),
        }
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
        let encoded_blob = Self::finalize_blobbing(&mut self.seed_buf, passphrase, config)?;

        Ok(KeyBlob {
            seed_type: self.seed_type.clone(),
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
    use crate::key_bundle::tests::*;
    use holochain_sodium::pwhash;

    #[test]
    fn it_should_blob_keybundle() {
        let mut seed = test_generate_random_seed();
        let mut passphrase = test_generate_random_seed();

        let mut bundle = KeyBundle::new_from_seed_buf(&mut seed, SeedType::Mock).unwrap();

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
        let mut passphrase = test_generate_random_seed();
        let mut seed_buf = test_generate_random_seed();
        let mut initial_seed = Seed::new(seed_buf, SeedType::Root);

        let blob = initial_seed
            .as_blob(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let typed_seed = Seed::from_blob(&blob, &mut passphrase, TEST_CONFIG).unwrap();

        match typed_seed {
            TypedSeed::Root(mut root_seed) => {
                assert_eq!(
                    0,
                    root_seed
                        .seed_mut()
                        .seed_buf
                        .compare(&mut initial_seed.seed_buf)
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn it_should_blob_device_pin_seed() {
        let mut passphrase = test_generate_random_seed();
        let mut seed_buf = test_generate_random_seed();
        let mut initial_device_pin_seed = DevicePinSeed::new(seed_buf);

        let blob = initial_device_pin_seed
            .as_blob(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let typed_seed = Seed::from_blob(&blob, &mut passphrase, TEST_CONFIG).unwrap();

        match s {
            TypedSeed::DevicePinSeed(mut device_pin_seed) => {
                assert_eq!(
                    0,
                    device_pin_seed
                        .seed_mut()
                        .seed_buf
                        .compare(&mut initial_seed.seed_buf)
                );
            }
            _ => unreachable!(),
        }
    }
}
