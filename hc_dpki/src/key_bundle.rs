#![allow(warnings)]
extern crate holochain_sodium;
use crate::keypair::holochain_sodium::{kx, secbuf::SecBuf, sign};
use holochain_sodium::random::random_secbuf;
use holochain_sodium::secbuf::SecBuf;
use holochain_sodium::*;

use crate::{
    keypair::*,
    util::{self, PwHashConfig},
};
use holochain_core_types::error::{HcResult, HolochainError};
use rustc_serialize::json;
use std::str;

use serde_derive::{Deserialize, Serialize};

const BLOB_FORMAT_VERSION = 2;

const BLOB_DATA_LEN_MISALIGN: usize = 1 // version byte
    + sign::PUBLICKEYBYTES
    + kx::PUBLICKEYBYTES
    + sign::SECRETKEYBYTES
    + kx::SECRETKEYBYTES;

#[allow(dead_code)]
pub const BLOB_DATA_LEN: usize = ((BLOB_DATA_LEN_MISALIGN + 8 - 1) / 8) * 8;


/// This struct is the bundle for the Key pairs. i.e. signing and encryption keys
///
/// The bundle_type tells if the bundle is a RootSeed bundle | DeviceSeed bundle | DevicePINSeed Bundle | ApplicationKeys Bundle
///
/// the data includes a base64 encoded string of the ReturnBundleData Struct that was created by combining all the keys in one SecBuf
#[derive(Serialize, Deserialize)]
pub struct KeyBlob {
    pub key_type: SeedType,
    pub hint: String,
    // encoded / serialized?
    pub data: String,
}

/// This struct type is for the return type for  util::pw_enc
#[derive(RustcDecodable, RustcEncodable)]
pub struct ReturnBlobData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub cipher: Vec<u8>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SeedType {
    Root,
    Revocation,
    Device,
    DevicePin,
    Application,
}

pub struct KeyBundle {
    pub sign_keys: SigningKeyPair,
    pub enc_keys: EncryptingKeyPair,
    pub seed_type: SeedType,
}

impl KeyBundle {
    /// Derive the keys from a 32 bytes seed buffer
    /// @param {SecBuf} seed - the seed buffer
    pub fn new_from_seed(seed: &mut SecBuf, seed_type: SeedType) -> Result<Self, HolochainError> {
        assert_eq!(seed.len(), SEED_SIZE);
        Ok(KeyBundle {
            sign_keys: SigningKeyPair::new_with_seed(seed)?,
            enc_keys: EncryptingKeyPair::new_with_seed(seed)?,
            seed_type,
        })
    }

    /// get the identifier key
    pub fn get_id(&self) -> Base32 {
        self.sign_keys.public
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
        // decoding the blob.data of type util::ReturnBlobData
        // Decode
        let blob_decoded = base64::decode(&blob.data)?;
        // Deserialize
        let blob_string = str::from_utf8(&blob_decoded).unwrap();
        let data: bundle::ReturnBundleData = json::decode(&blob_string).unwrap();
        // Decrypt
        let mut decrypted_data = SecBuf::with_secure(BUNDLE_DATA_LEN);
        util::pw_dec(&data, passphrase, &mut decrypted_data, config)?;

        let mut priv_sign = SecBuf::with_secure(SIGNATURE_SIZE);
        let mut priv_enc = SecBuf::with_secure(32);
        let mut pub_sign = SecBuf::with_secure(SIGNATURE_SIZE);
        let mut pub_enc = SecBuf::with_secure(32);

        // FIXME
//        let pub_keys = {
//            let decrypted_data = decrypted_data.read_lock();
//            if decrypted_data[0] != BLOB_FORMAT_VERSION {
//                return Err(HolochainError::ErrorGeneric(format!(
//                    "Invalid Blob Format: v{:?} != v{:?}",
//                    decrypted_data[0], BLOB_FORMAT_VERSION
//                )));
//            }
//            priv_sign.write(0, &decrypted_data[65..129])?;
//            priv_enc.write(0, &decrypted_data[129..161])?;
//
//            KeyBuffer::with_raw_parts(
//                array_ref![&decrypted_data, 1, 32],
//                array_ref![&decrypted_data, 33, 32],
//            )
//                .render()
//        };

        Ok(KeyBundle {
            sign_keys: SigningKeyPair::new(pub_sign, priv_sign),
            enc_keys: EncryptingKeyPair::new(pub_enc, priv_enc),
            seed_type: blob.seed_type.clone(),
        })
    }

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
        // let corrected_pub_keys = KeyBuffer::with_corrected(&self.pub_keys)?;
        // Initialize buffer
        let mut data_buf = SecBuf::with_secure(BUNDLE_DATA_LEN);
        let mut offset: usize = 0;
        // Write version
        data_buf.write(offset, &[BLOB_FORMAT_VERSION])?;
        offset += 1;
        // Write public signing key
        data_buf.write(1, corrected_pub_keys.get_sig())?;
        offset += sign::PUBLICKEYBYTES;
        // Write public signing key
        data_buf.write(offset, corrected_pub_keys.get_enc())?;
        offset += kx::PUBLICKEYBYTES;
        // Write public signing key
        data_buf.write(offset, &**self.sign_priv.read_lock())?;
        offset += sign::SECRETKEYBYTES;
        // Write public signing key
        data_buf.write(offset, &**self.enc_priv.read_lock())?;
        // encrypt buffer
        let encrypted_blob = util::pw_enc(&mut data_buf, passphrase, config)?;
        let serialized_blob = json::encode(&encrypted_blob).expect("");
        // conver to base64
        let encoded_blob = base64::encode(&serialized_blob);
        // Done
        Ok(KeyBlob {
            key_type: self.seed_type.clone(),
            hint,
            data: encoded_blob,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_sodium::{pwhash, random::random_secbuf};

    const TEST_CONFIG: Option<PwHashConfig> = Some(PwHashConfig(
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
        pwhash::ALG_ARGON2ID13,
    ));

    #[test]
    fn it_should_set_keypair_from_seed() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);

        let keypair = KeyBundle::new_from_seed(&mut seed).unwrap();

        assert_eq!(64, keypair.sign_priv.len());
        assert_eq!(32, keypair.enc_priv.len());
    }

    #[test]
    fn it_should_get_id() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let keypair = KeyBundle::new_from_seed(&mut seed).unwrap();

        let pk: String = keypair.get_id();
        println!("pk: {:?}", pk);
        let pk1: String = keypair.get_id();
        println!("pk1: {:?}", pk1);
        assert_eq!(pk, pk1);
    }

    #[test]
    fn it_should_sign_message_and_verify() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let mut keypair = KeyBundle::new_from_seed(&mut seed).unwrap();

        let mut message = SecBuf::with_insecure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_insecure(SIGNATURE_SIZE);

        keypair.sign(&mut message, &mut message_signed).unwrap();

        let check: i32 =
            KeyBundle::verify(keypair.pub_keys, &mut message_signed, &mut message).unwrap();
        assert_eq!(0, check);
    }

    #[test]
    fn it_should_invalidate_altered_message() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let mut keypair = KeyBundle::new_from_seed(&mut seed).unwrap();

        let mut message = SecBuf::with_insecure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_insecure(SIGNATURE_SIZE);

        keypair.sign(&mut message, &mut message_signed).unwrap();

        random_secbuf(&mut message);

        let check: i32 =
            KeyBundle::verify(keypair.pub_keys, &mut message_signed, &mut message).unwrap();
        assert_ne!(0, check);
    }

    // #[test]
    // fn it_should_encode_n_decode_data() {
    //     let mut seed = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed);
    //     let mut keypair_main = Keypair::new_from_seed(&mut seed).unwrap();
    //
    //     let mut seed_1 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_1);
    //     let mut keypair_1 = Keypair::new_from_seed(&mut seed_1).unwrap();
    //
    //     let mut message = SecBuf::with_insecure(16);
    //     random_secbuf(&mut message);
    //
    //     let recipient_id = vec![&keypair_1.pub_keys];
    //
    //     let mut out = Vec::new();
    //     keypair_main
    //         .encrypt(recipient_id, &mut message, &mut out)
    //         .unwrap();
    //
    //     match keypair_1.decrypt(keypair_main.pub_keys, &mut out) {
    //         Ok(mut dm) => {
    //             let message = message.read_lock();
    //             let dm = dm.read_lock();
    //             assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
    //         }
    //         Err(_) => {
    //             assert!(false);
    //         }
    //     };
    // }
    //
    // #[test]
    // fn it_should_encode_n_decode_data_for_multiple_users2() {
    //     let mut seed = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed);
    //     let mut keypair_main = Keypair::new_from_seed(&mut seed).unwrap();
    //
    //     let mut seed_1 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_1);
    //     let keypair_1 = Keypair::new_from_seed(&mut seed_1).unwrap();
    //
    //     let mut seed_2 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_2);
    //     let mut keypair_2 = Keypair::new_from_seed(&mut seed_2).unwrap();
    //
    //     let mut message = SecBuf::with_insecure(16);
    //     random_secbuf(&mut message);
    //
    //     let recipient_id = vec![&keypair_1.pub_keys, &keypair_2.pub_keys];
    //
    //     let mut out = Vec::new();
    //     keypair_main
    //         .encrypt(recipient_id, &mut message, &mut out)
    //         .unwrap();
    //
    //     match keypair_2.decrypt(keypair_main.pub_keys, &mut out) {
    //         Ok(mut dm) => {
    //             let message = message.read_lock();
    //             let dm = dm.read_lock();
    //             assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
    //         }
    //         Err(_) => {
    //             assert!(false);
    //         }
    //     };
    // }
    //
    // #[test]
    // fn it_should_encode_n_decode_data_for_multiple_users1() {
    //     let mut seed = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed);
    //     let mut keypair_main = Keypair::new_from_seed(&mut seed).unwrap();
    //
    //     let mut seed_1 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_1);
    //     let mut keypair_1 = Keypair::new_from_seed(&mut seed_1).unwrap();
    //
    //     let mut seed_2 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_2);
    //     let keypair_2 = Keypair::new_from_seed(&mut seed_2).unwrap();
    //
    //     let mut message = SecBuf::with_insecure(16);
    //     random_secbuf(&mut message);
    //
    //     let recipient_id = vec![&keypair_1.pub_keys, &keypair_2.pub_keys];
    //
    //     let mut out = Vec::new();
    //     keypair_main
    //         .encrypt(recipient_id, &mut message, &mut out)
    //         .unwrap();
    //
    //     match keypair_1.decrypt(keypair_main.pub_keys, &mut out) {
    //         Ok(mut dm) => {
    //             println!("Decrypted Message: {:?}", dm);
    //             let message = message.read_lock();
    //             let dm = dm.read_lock();
    //             assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
    //         }
    //         Err(_) => {
    //             println!("Error");
    //             assert!(false);
    //         }
    //     };
    // }
    //
    // #[test]
    // fn it_should_with_fail_when_wrong_key_used_to_decrypt() {
    //     let mut seed = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed);
    //     let mut keypair_main = Keypair::new_from_seed(&mut seed).unwrap();
    //
    //     let mut seed_1 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_1);
    //     let keypair_1 = Keypair::new_from_seed(&mut seed_1).unwrap();
    //
    //     let mut seed_2 = SecBuf::with_insecure(SEEDSIZE);
    //     random_secbuf(&mut seed_2);
    //     let mut keypair_2 = Keypair::new_from_seed(&mut seed_2).unwrap();
    //
    //     let mut message = SecBuf::with_insecure(16);
    //     random_secbuf(&mut message);
    //
    //     let recipient_id = vec![&keypair_1.pub_keys];
    //
    //     let mut out = Vec::new();
    //     keypair_main
    //         .encrypt(recipient_id, &mut message, &mut out)
    //         .unwrap();
    //
    //     keypair_2
    //         .decrypt(keypair_main.pub_keys, &mut out)
    //         .expect_err("should have failed");
    // }

    #[test]
    fn it_should_get_from_bundle() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let mut keypair = KeyBundle::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut passphrase);

        let blob = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let keypair_from_bundle =
            KeyBundle::from_bundle(&blob, &mut passphrase, TEST_CONFIG).unwrap();

        assert_eq!(64, keypair_from_bundle.sign_priv.len());
        assert_eq!(32, keypair_from_bundle.enc_priv.len());
        assert_eq!(92, keypair_from_bundle.pub_keys.len());
    }

    #[test]
    fn it_should_get_bundle() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let mut keypair = KeyBundle::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut passphrase);

        let blob = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        println!("Bundle.bundle_type: {}", blob.bundle_type);
        println!("Bundle.Hint: {}", blob.hint);
        println!("Bundle.data: {}", blob.data);

        assert_eq!("hint", blob.hint);
    }

    #[test]
    fn it_should_try_get_bundle_and_decode_it() {
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut seed);
        let mut keypair = KeyBundle::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEED_SIZE);
        random_secbuf(&mut passphrase);

        let blob = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        println!("Bundle.bundle_type: {}", blob.bundle_type);
        println!("Bundle.Hint: {}", blob.hint);
        println!("Bundle.data: {}", blob.data);

        let keypair_from_bundle =
            KeyBundle::from_bundle(&blob, &mut passphrase, TEST_CONFIG).unwrap();

        assert_eq!(64, keypair_from_bundle.sign_priv.len());
        assert_eq!(32, keypair_from_bundle.enc_priv.len());
        assert_eq!(92, keypair_from_bundle.pub_keys.len());
    }
}
