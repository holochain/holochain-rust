#![allow(warnings)]
use holochain_sodium::{kx, secbuf::SecBuf, sign, *};

use crate::{
    key_bundle::*,
    keypair::*,
    password_encryption::{self, EncryptedData, PwHashConfig},
    utils, SEED_SIZE,
};
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
};
use rustc_serialize::json;
use std::str;

use serde_derive::{Deserialize, Serialize};

const BLOB_FORMAT_VERSION: u8 = 2;

const BLOB_DATA_LEN_MISALIGN: usize = 1 // version byte
    + sign::PUBLICKEYBYTES
    + kx::PUBLICKEYBYTES
    + sign::SECRETKEYBYTES
    + kx::SECRETKEYBYTES;

pub const BLOB_DATA_LEN: usize = ((BLOB_DATA_LEN_MISALIGN + 8 - 1) / 8) * 8;

/// The data includes a base64 encoded, json serialized string of the EncryptedData that
/// was created by concatenating all the keys in one SecBuf
#[derive(Serialize, Deserialize)]
pub struct KeyBlob {
    pub seed_type: SeedType,
    pub hint: String,
    ///  base64 encoded, json serialized string of the EncryptedData
    pub data: String,
}

impl KeyBundle {
    /// Generate an encrypted blob for persistence
    /// @param {SecBuf} passphrase - the encryption passphrase
    /// @param {string} hint - additional info / description for the bundle
    /// @param {string} config - Settings for pwhash
    pub fn as_blob(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBlob> {
        // Initialize buffer
        let mut data_buf = SecBuf::with_secure(BLOB_DATA_LEN);
        let mut offset: usize = 0;
        // Write version
        data_buf.write(offset, &[BLOB_FORMAT_VERSION])?;
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
        assert_eq!(offset, BLOB_DATA_LEN_MISALIGN);

        // encrypt buffer
        let encrypted_blob = password_encryption::pw_enc(&mut data_buf, passphrase, config)?;
        let serialized_blob = json::encode(&encrypted_blob).expect("Failed to serialize KeyBundle");
        // conver to base64
        let encoded_blob = base64::encode(&serialized_blob);
        // Done
        Ok(KeyBlob {
            seed_type: self.seed_type.clone(),
            hint,
            data: encoded_blob,
        })
    }

    /// Construct the pairs from an encrypted blob
    /// @param {object} bundle - persistence info
    /// @param {SecBuf} passphrase - decryption passphrase
    /// @param {string} config - Settings for pwhash
    pub fn from_blob(
        blob: &KeyBlob,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> HcResult<KeyBundle> {
        // decoding the blob.data of type EncryptedData
        let blob_decoded = base64::decode(&blob.data)?;

        // Deserialize
        let blob_string = str::from_utf8(&blob_decoded).unwrap();
        let data: EncryptedData = json::decode(&blob_string).unwrap();
        // Decrypt
        let mut decrypted_data = SecBuf::with_secure(BLOB_DATA_LEN);
        password_encryption::pw_dec(&data, passphrase, &mut decrypted_data, config)?;

        let mut pub_sign = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut pub_enc = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
        let mut priv_sign = SecBuf::with_secure(sign::SECRETKEYBYTES);
        let mut priv_enc = SecBuf::with_secure(kx::SECRETKEYBYTES);

        {
            let decrypted_data = decrypted_data.read_lock();
            if decrypted_data[0] != BLOB_FORMAT_VERSION {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Invalid Blob Format: v{:?} != v{:?}",
                    decrypted_data[0], BLOB_FORMAT_VERSION
                )));
            }
            pub_sign.write(0, &decrypted_data[1..33])?;
            pub_enc.write(0, &decrypted_data[33..65])?;
            priv_sign.write(0, &decrypted_data[65..129])?;
            priv_enc.write(0, &decrypted_data[129..161])?;
        }

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

        let mut bundle = KeyBundle::new_from_seed(&mut seed, SeedType::Mock).unwrap();

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
}
