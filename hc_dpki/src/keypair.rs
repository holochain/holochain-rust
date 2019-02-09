#![allow(warnings)]
extern crate holochain_sodium;
use crate::keypair::holochain_sodium::secbuf::SecBuf;
use crate::keypair::holochain_sodium::kx;
use crate::keypair::holochain_sodium::sign;
use holochain_sodium::random::random_secbuf;

use crate::{
    bundle,
    util::{self, PwHashConfig},
};
use holochain_core_types::{agent::KeyBuffer, error::HolochainError};
use rustc_serialize::json;
use std::str;

pub struct Keypair {
    pub pub_keys: String,
    pub sign_priv: SecBuf,
    pub enc_priv: SecBuf,
}

pub const SEEDSIZE: usize = 32;
pub const SIGNATURESIZE: usize = 64;

const BUNDLE_DATA_LEN_MISALIGN: usize = 1 // version byte
    + sign::PUBLICKEYBYTES
    + kx::PUBLICKEYBYTES
    + sign::SECRETKEYBYTES
    + kx::SECRETKEYBYTES;

pub const BUNDLE_DATA_LEN: usize = ((BUNDLE_DATA_LEN_MISALIGN + 8 - 1) / 8) * 8;

impl Keypair {
    /// derive the pairs from a 32 byte seed buffer
    ///
    /// @param {SecBuf} seed - the seed buffer
    pub fn new_from_seed(seed: &mut SecBuf) -> Result<Self, HolochainError> {
        let mut sign_public_key = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut sign_secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        let mut enc_public_key = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
        let mut enc_secret_key = SecBuf::with_secure(kx::SECRETKEYBYTES);

        sign::seed_keypair(&mut sign_public_key, &mut sign_secret_key, seed)?;
        kx::seed_keypair(seed, &mut enc_public_key, &mut enc_secret_key)?;

        Ok(Keypair {
            pub_keys: util::encode_id(&mut sign_public_key, &mut enc_public_key),
            sign_priv: sign_secret_key,
            enc_priv: enc_secret_key,
        })
    }

    /// get the keypair identifier string
    ///
    /// @return {string}
    pub fn get_id(&self) -> String {
        return self.pub_keys.clone();
    }

    /// generate an encrypted persistence bundle
    ///
    /// @param {SecBuf} passphrase - the encryption passphrase
    ///
    /// @param {string} hint - additional info / description for the bundle
    pub fn get_bundle(
        &mut self,
        passphrase: &mut SecBuf,
        hint: String,
        config: Option<PwHashConfig>,
    ) -> Result<bundle::KeyBundle, HolochainError> {
        let bundle_type: String = "hcKeypair".to_string();
        let corrected_pub_keys = KeyBuffer::with_corrected(&self.pub_keys)?;

        let mut key_buf = SecBuf::with_secure(BUNDLE_DATA_LEN);

        let mut offset: usize = 0;

        key_buf.write(offset, &[1])?;
        offset += 1;

        key_buf.write(1, corrected_pub_keys.get_sig())?;
        offset += sign::PUBLICKEYBYTES;

        key_buf.write(offset, corrected_pub_keys.get_enc())?;
        offset += kx::PUBLICKEYBYTES;

        key_buf.write(offset, &**self.sign_priv.read_lock())?;
        offset += sign::SECRETKEYBYTES;

        key_buf.write(offset, &**self.enc_priv.read_lock())?;

        let password_encrypted: bundle::ReturnBundleData =
            util::pw_enc(&mut key_buf, passphrase, config)?;
        let bundle_data_serialized = json::encode(&password_encrypted).unwrap();

        // conver to base64
        let bundle_data_encoded = base64::encode(&bundle_data_serialized);

        Ok(bundle::KeyBundle {
            bundle_type,
            hint,
            data: bundle_data_encoded,
        })
    }

    /// initialize the pairs from an encrypted persistence bundle
    ///
    /// @param {object} bundle - persistence info
    ///
    /// @param {SecBuf} passphrase - decryption passphrase
    pub fn from_bundle(
        bundle: &bundle::KeyBundle,
        passphrase: &mut SecBuf,
        config: Option<PwHashConfig>,
    ) -> Result<Keypair, HolochainError> {
        // decoding the bundle.data of type util::ReturnBundledata
        let bundle_decoded = base64::decode(&bundle.data)?;
        let bundle_string = str::from_utf8(&bundle_decoded).unwrap();
        let data: bundle::ReturnBundleData = json::decode(&bundle_string).unwrap();
        let mut decrypted_data = SecBuf::with_secure(BUNDLE_DATA_LEN);
        util::pw_dec(&data, passphrase, &mut decrypted_data, config)?;
        let mut sign_priv = SecBuf::with_secure(SIGNATURESIZE);
        let mut enc_priv = SecBuf::with_secure(32);

        let pub_keys = {
            let decrypted_data = decrypted_data.read_lock();
            if decrypted_data[0] != 1 {
                return Err(HolochainError::ErrorGeneric(format!(
                    "Invalid Bundle Version : {:?}",
                    decrypted_data[0]
                )));
            }
            sign_priv.write(0, &decrypted_data[65..129])?;
            enc_priv.write(0, &decrypted_data[129..161])?;

            KeyBuffer::with_raw_parts(
                array_ref![&decrypted_data, 1, 32],
                array_ref![&decrypted_data, 33, 32],
            )
            .render()
        };

        Ok(Keypair {
            pub_keys,
            enc_priv,
            sign_priv,
        })
    }

    /// sign some arbitrary data with the signing private key
    ///
    /// @param {SecBuf} data - the data to sign
    ///
    /// @param {SecBuf} signature - Empty Buf the sign
    pub fn sign(
        &mut self,
        data: &mut SecBuf,
        signature: &mut SecBuf,
    ) -> Result<(), HolochainError> {
        sign::sign(data, &mut self.sign_priv, signature)?;
        Ok(())
    }

    /// verify data that was signed with our private signing key
    ///
    /// @param {SecBuf} signature
    ///
    /// @param {SecBuf} data
    pub fn verify(
        pub_keys: String,
        signature: &mut SecBuf,
        data: &mut SecBuf,
    ) -> Result<i32, HolochainError> {
        let mut sign_pub = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut enc_pub = SecBuf::with_insecure(kx::PUBLICKEYBYTES);

        util::decode_id(pub_keys, &mut sign_pub, &mut enc_pub)?;
        let verified: i32 = sign::verify(signature, data, &mut sign_pub);
        Ok(verified)
    }

    // /// encrypt arbitrary data to be readale by potentially multiple recipients
    // ///
    // /// @param {array<string>} recipientIds - multiple recipient identifier strings
    // ///
    // /// @param {Buffer} data - the data to encrypt
    // ///
    // /// @param {Buffer} out - Empty vec[secBuf]
    // pub fn encrypt(
    //     &mut self,
    //     recipient_id: Vec<&String>,
    //     data: &mut SecBuf,
    //     out: &mut Vec<SecBuf>,
    // ) -> Result<(), HolochainError> {
    //     let mut sym_secret = SecBuf::with_secure(32);
    //     random_secbuf(&mut sym_secret);
    //
    //     let mut server_rx = SecBuf::with_insecure(kx::SESSIONKEYBYTES);
    //     let mut server_tx = SecBuf::with_insecure(kx::SESSIONKEYBYTES);
    //
    //     let pub_keys = &mut self.pub_keys;
    //     let mut sign_pub = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    //     let mut enc_pub = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
    //     util::decode_id(pub_keys.to_string(), &mut sign_pub, &mut enc_pub)?;
    //
    //     for client_pk in recipient_id {
    //         let mut client_sign_pub = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    //         let mut client_enc_pub = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
    //
    //         util::decode_id(
    //             client_pk.to_string(),
    //             &mut client_sign_pub,
    //             &mut client_enc_pub,
    //         )?;
    //
    //         kx::server_session(
    //             &mut enc_pub,
    //             &mut self.enc_priv,
    //             &mut client_enc_pub,
    //             &mut server_rx,
    //             &mut server_tx,
    //         )?;
    //
    //         let mut nonce = SecBuf::with_insecure(16);
    //         random_secbuf(&mut nonce);
    //         let mut cipher = SecBuf::with_insecure(sym_secret.len() + aead::ABYTES);
    //
    //         aead::enc(
    //             &mut sym_secret,
    //             &mut server_tx,
    //             None,
    //             &mut nonce,
    //             &mut cipher,
    //         )?;
    //         out.push(nonce);
    //         out.push(cipher);
    //     }
    //
    //     let mut nonce = SecBuf::with_insecure(16);
    //     random_secbuf(&mut nonce);
    //     let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
    //     let mut data = data;
    //     aead::enc(&mut data, &mut sym_secret, None, &mut nonce, &mut cipher)?;
    //     out.push(nonce);
    //     out.push(cipher);
    //     Ok(())
    // }
    //
    // /// attempt to decrypt the cipher buffer (assuming it was targeting us)
    // ///
    // /// @param {string} sourceId - identifier string of who encrypted this data
    // ///
    // /// @param {Buffer} cipher - the encrypted data
    // ///
    // /// @return {Result<SecBuf,String>} - the decrypted data
    // pub fn decrypt(
    //     &mut self,
    //     source_id: String,
    //     cipher_bundle: &mut Vec<SecBuf>,
    // ) -> Result<SecBuf, HolochainError> {
    //     let mut source_sign_pub = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    //     let mut source_enc_pub = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
    //     util::decode_id(source_id, &mut source_sign_pub, &mut source_enc_pub)?;
    //
    //     let client_pub_keys = &self.pub_keys;
    //     let mut client_sign_pub = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    //     let mut client_enc_pub = SecBuf::with_insecure(kx::PUBLICKEYBYTES);
    //     util::decode_id(
    //         client_pub_keys.to_string(),
    //         &mut client_sign_pub,
    //         &mut client_enc_pub,
    //     )?;
    //     let mut client_enc_priv = &mut self.enc_priv;
    //
    //     let mut client_rx = SecBuf::with_insecure(kx::SESSIONKEYBYTES);
    //     let mut client_tx = SecBuf::with_insecure(kx::SESSIONKEYBYTES);
    //     kx::client_session(
    //         &mut client_enc_pub,
    //         &mut client_enc_priv,
    //         &mut source_enc_pub,
    //         &mut client_rx,
    //         &mut client_tx,
    //     )?;
    //
    //     let mut sys_secret_check: Option<SecBuf> = None;
    //
    //     while cipher_bundle.len() != 2 {
    //         let mut nonce: Vec<_> = cipher_bundle.splice(..1, vec![]).collect();
    //         let mut cipher: Vec<_> = cipher_bundle.splice(..1, vec![]).collect();
    //         let mut nonce = &mut nonce[0];
    //         let mut cipher = &mut cipher[0];
    //         let mut sys_secret = SecBuf::with_insecure(cipher.len() - aead::ABYTES);
    //
    //         match aead::dec(
    //             &mut sys_secret,
    //             &mut client_rx,
    //             None,
    //             &mut nonce,
    //             &mut cipher,
    //         ) {
    //             Ok(_) => {
    //                 if util::check_if_wrong_secbuf(&mut sys_secret) {
    //                     sys_secret_check = Some(sys_secret);
    //                     break;
    //                 } else {
    //                     sys_secret_check = None;
    //                 }
    //             }
    //             Err(_) => {
    //                 sys_secret_check = None;
    //             }
    //         };
    //     }
    //
    //     let mut cipher: Vec<_> = cipher_bundle
    //         .splice(cipher_bundle.len() - 1.., vec![])
    //         .collect();
    //     let mut nonce: Vec<_> = cipher_bundle
    //         .splice(cipher_bundle.len() - 1.., vec![])
    //         .collect();
    //     let mut nonce = &mut nonce[0];
    //     let mut cipher = &mut cipher[0];
    //     let mut decrypeted_message = SecBuf::with_insecure(cipher.len() - aead::ABYTES);
    //
    //     if let Some(mut secret) = sys_secret_check {
    //         aead::dec(
    //             &mut decrypeted_message,
    //             &mut secret,
    //             None,
    //             &mut nonce,
    //             &mut cipher,
    //         )?;
    //         Ok(decrypeted_message)
    //     } else {
    //         Err(HolochainError::new(
    //             &"could not decrypt - not a recipient?".to_string(),
    //         ))
    //     }
    // }
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
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);

        let keypair = Keypair::new_from_seed(&mut seed).unwrap();

        assert_eq!(64, keypair.sign_priv.len());
        assert_eq!(32, keypair.enc_priv.len());
    }

    #[test]
    fn it_should_get_id() {
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let keypair = Keypair::new_from_seed(&mut seed).unwrap();

        let pk: String = keypair.get_id();
        println!("pk: {:?}", pk);
        let pk1: String = keypair.get_id();
        println!("pk1: {:?}", pk1);
        assert_eq!(pk, pk1);
    }

    #[test]
    fn it_should_sign_message_and_verify() {
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();

        let mut message = SecBuf::with_insecure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_insecure(SIGNATURESIZE);

        keypair.sign(&mut message, &mut message_signed).unwrap();

        let check: i32 =
            Keypair::verify(keypair.pub_keys, &mut message_signed, &mut message).unwrap();
        assert_eq!(0, check);
    }

    #[test]
    fn it_should_invalidate_altered_message() {
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();

        let mut message = SecBuf::with_insecure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_insecure(SIGNATURESIZE);

        keypair.sign(&mut message, &mut message_signed).unwrap();

        random_secbuf(&mut message);

        let check: i32 =
            Keypair::verify(keypair.pub_keys, &mut message_signed, &mut message).unwrap();
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
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut passphrase);

        let bundle: bundle::KeyBundle = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        let keypair_from_bundle =
            Keypair::from_bundle(&bundle, &mut passphrase, TEST_CONFIG).unwrap();

        assert_eq!(64, keypair_from_bundle.sign_priv.len());
        assert_eq!(32, keypair_from_bundle.enc_priv.len());
        assert_eq!(92, keypair_from_bundle.pub_keys.len());
    }

    #[test]
    fn it_should_get_bundle() {
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut passphrase);

        let bundle: bundle::KeyBundle = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        println!("Bundle.bundle_type: {}", bundle.bundle_type);
        println!("Bundle.Hint: {}", bundle.hint);
        println!("Bundle.data: {}", bundle.data);

        assert_eq!("hint", bundle.hint);
    }

    #[test]
    fn it_should_try_get_bundle_and_decode_it() {
        let mut seed = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();
        let mut passphrase = SecBuf::with_insecure(SEEDSIZE);
        random_secbuf(&mut passphrase);

        let bundle: bundle::KeyBundle = keypair
            .get_bundle(&mut passphrase, "hint".to_string(), TEST_CONFIG)
            .unwrap();

        println!("Bundle.bundle_type: {}", bundle.bundle_type);
        println!("Bundle.Hint: {}", bundle.hint);
        println!("Bundle.data: {}", bundle.data);

        let keypair_from_bundle =
            Keypair::from_bundle(&bundle, &mut passphrase, TEST_CONFIG).unwrap();

        assert_eq!(64, keypair_from_bundle.sign_priv.len());
        assert_eq!(32, keypair_from_bundle.enc_priv.len());
        assert_eq!(92, keypair_from_bundle.pub_keys.len());
    }
}
