use crate::{
    holochain_sodium::{aead, kx, random::random_secbuf, secbuf::SecBuf, sign},
    util,
};

pub const SEEDSIZE: usize = 32 as usize;

pub struct Keypair {
    pub_keys: SecBuf,
    sign_priv: SecBuf,
    enc_priv: SecBuf,
}

impl Keypair {
    ///
    /// derive the pairs from a 32 byte seed buffer
    ///  
    /// @param {SecBuf} seed - the seed buffer
    pub fn new_from_seed(seed: &mut SecBuf) -> Self {
        let mut seed = seed;
        let mut sign_public_key = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut sign_secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        let mut enc_public_key = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        let mut enc_secret_key = SecBuf::with_secure(kx::SECRETKEYBYTES);

        sign::seed_keypair(&mut sign_public_key, &mut sign_secret_key, &mut seed).unwrap();
        kx::seed_keypair(&mut seed, &mut enc_public_key, &mut enc_secret_key).unwrap();

        let mut pub_id = SecBuf::with_secure(sign::PUBLICKEYBYTES + kx::PUBLICKEYBYTES);
        util::encode_id(&mut sign_public_key, &mut enc_public_key, &mut pub_id);

        Keypair {
            pub_keys: pub_id,
            sign_priv: sign_secret_key,
            enc_priv: enc_secret_key,
        }
    }

    /// generate an encrypted persistence bundle
    ///
    /// @param {SecBuf} passphrase - the encryption passphrase
    ///
    /// @param {string} hint - additional info / description for the bundle
    pub fn get_bundle(&mut self, passphrase: &mut SecBuf, hint: String) -> util::Bundle {
        let mut passphrase = passphrase;
        let bundle_type: String = "hcKeypair".to_string();
        let pw_pub_keys: util::ReturnBundleData = util::pw_enc(&mut self.pub_keys, &mut passphrase);
        let pw_sign_priv: util::ReturnBundleData =
            util::pw_enc(&mut self.sign_priv, &mut passphrase);
        let pw_enc_priv: util::ReturnBundleData = util::pw_enc(&mut self.enc_priv, &mut passphrase);

        return util::Bundle {
            bundle_type,
            hint,
            data: util::Keys {
                pw_pub_keys,
                pw_sign_priv,
                pw_enc_priv,
            },
        };
    }

    /// initialize the pairs from an encrypted persistence bundle
    ///
    /// @param {object} bundle - persistence info
    ///
    /// @param {SecBuf} passphrase - decryption passphrase
    pub fn from_bundle(bundle: &util::Bundle, passphrase: &mut SecBuf) -> Keypair {
        let pk: &util::ReturnBundleData = &bundle.data.pw_pub_keys;
        let epk: &util::ReturnBundleData = &bundle.data.pw_enc_priv;
        let spk: &util::ReturnBundleData = &bundle.data.pw_sign_priv;
        let pub_keys = util::pw_dec(pk, passphrase);
        let enc_priv = util::pw_dec(epk, passphrase);
        let sign_priv = util::pw_dec(spk, passphrase);
        Keypair {
            pub_keys,
            enc_priv,
            sign_priv,
        }
    }

    /// get the keypair identifier string
    ///
    /// @return {SecBuf}
    pub fn get_id(&mut self) -> SecBuf {
        let mut id = SecBuf::with_secure(self.pub_keys.len());
        // let pub_keys = self.pub_keys.write_lock();
        {
            let mut id = id.write_lock();
            let pub_keys = self.pub_keys.read_lock();
            for i in 0..pub_keys.len() {
                id[i] = pub_keys[i].clone();
            }
        }
        return id;
    }

    /// sign some arbitrary data with the signing private key
    ///
    /// @param {SecBuf} data - the data to sign
    ///
    /// @param {SecBuf} signature - Empty Buf the sign
    pub fn sign(&mut self, data: &mut SecBuf, signature: &mut SecBuf) {
        let mut data = data;
        let mut signature = signature;
        let mut sign_priv = &mut self.sign_priv;
        sign::sign(&mut data, &mut sign_priv, &mut signature).unwrap();
    }

    /// verify data that was signed with our private signing key
    ///
    /// @param {SecBuf} signature
    ///
    /// @param {SecBuf} data
    pub fn verify(&mut self, signature: &mut SecBuf, data: &mut SecBuf) -> i32 {
        let mut data = data;
        let mut signature = signature;
        let mut pub_keys = &mut self.pub_keys;
        let mut sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);

        util::decode_id(&mut pub_keys, &mut sign_pub, &mut enc_pub);

        sign::verify(&mut signature, &mut data, &mut sign_pub)
    }

    /// encrypt arbitrary data to be readale by potentially multiple recipients
    ///
    /// @param {array<string>} recipientIds - multiple recipient identifier strings
    ///
    /// @param {Buffer} data - the data to encrypt
    ///
    /// @param {Buffer} out - Empty vec[secBuf]
    pub fn encrypt(
        &mut self,
        recipient_id: Vec<&mut SecBuf>,
        data: &mut SecBuf,
        out: &mut Vec<SecBuf>,
    ) {
        let mut sym_secret = SecBuf::with_secure(32);
        random_secbuf(&mut sym_secret);

        let mut srv_rx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        let mut srv_tx = SecBuf::with_secure(kx::SESSIONKEYBYTES);

        let mut pub_keys = &mut self.pub_keys;
        let mut sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        util::decode_id(&mut pub_keys, &mut sign_pub, &mut enc_pub);

        let mut enc_priv = &mut self.enc_priv;

        for client_pk in recipient_id {
            let mut r_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
            let mut r_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
            let mut client_pk = client_pk;

            util::decode_id(&mut client_pk, &mut r_sign_pub, &mut r_enc_pub);

            kx::server_session(
                &mut enc_pub,
                &mut enc_priv,
                &mut r_enc_pub,
                &mut srv_rx,
                &mut srv_tx,
            )
            .unwrap();

            let mut nonce = SecBuf::with_insecure(16);
            random_secbuf(&mut nonce);
            let mut cipher = SecBuf::with_insecure(sym_secret.len() + aead::ABYTES);

            aead::enc(&mut sym_secret, &mut srv_tx, None, &mut nonce, &mut cipher).unwrap();
            out.push(nonce);
            out.push(cipher);
        }

        let mut nonce = SecBuf::with_insecure(16);
        random_secbuf(&mut nonce);
        let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
        let mut data = data;
        aead::enc(&mut data, &mut sym_secret, None, &mut nonce, &mut cipher).unwrap();
        out.push(nonce);
        out.push(cipher);
    }

    /// attempt to decrypt the cipher buffer (assuming it was targeting us)
    ///
    /// @param {string} sourceId - identifier string of who encrypted this data
    ///
    /// @param {Buffer} cipher - the encrypted data
    ///
    /// @return {Result<SecBuf,String>} - the decrypted data
    pub fn decrypt(
        &mut self,
        source_id: &mut SecBuf,
        cipher_bundle: &mut Vec<SecBuf>,
    ) -> Result<SecBuf, String> {
        // let &mut cipher_bundle = bundle.iter().cloned();
        let mut source_id = source_id;
        let mut source_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut source_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        util::decode_id(&mut source_id, &mut source_sign_pub, &mut source_enc_pub);

        let mut client_pub_keys = &mut self.pub_keys;
        let mut client_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut client_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        util::decode_id(
            &mut client_pub_keys,
            &mut client_sign_pub,
            &mut client_enc_pub,
        );
        let mut client_enc_priv = &mut self.enc_priv;

        let mut cli_rx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        let mut cli_tx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        kx::client_session(
            &mut client_enc_pub,
            &mut client_enc_priv,
            &mut source_enc_pub,
            &mut cli_rx,
            &mut cli_tx,
        )
        .unwrap();

        let mut sys_secret_check: Option<SecBuf> = None;

        while cipher_bundle.len() != 2 {
            println!("Round trip");
            let mut n: Vec<_> = cipher_bundle.splice(..1, vec![]).collect();
            let mut c: Vec<_> = cipher_bundle.splice(..1, vec![]).collect();
            let mut n = &mut n[0];
            let mut c = &mut c[0];
            let mut sys_secret = SecBuf::with_insecure(c.len() - aead::ABYTES);

            match aead::dec(&mut sys_secret, &mut cli_rx, None, &mut n, &mut c) {
                Ok(_) => {
                    if util::check_if_wrong_secbuf(&mut sys_secret) {
                        println!("TRUE");
                        sys_secret_check = Some(sys_secret);
                        break;
                    } else {
                        println!("FALSE");

                        sys_secret_check = None;
                    }
                }
                Err(_) => {
                    sys_secret_check = None;
                }
            };
        }

        let mut c: Vec<_> = cipher_bundle
            .splice(cipher_bundle.len() - 1.., vec![])
            .collect();
        let mut n: Vec<_> = cipher_bundle
            .splice(cipher_bundle.len() - 1.., vec![])
            .collect();
        let mut n = &mut n[0];
        let mut c = &mut c[0];
        let mut dm = SecBuf::with_insecure(c.len() - aead::ABYTES);

        if let Some(mut secret) = sys_secret_check {
            aead::dec(&mut dm, &mut secret, None, &mut n, &mut c).unwrap();
            Ok(dm)
        } else {
            Err("could not decrypt - not a recipient?".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;

    #[test]
    fn it_should_get_from_bundle() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed);
        let mut passphrase = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut passphrase);

        let bundle: util::Bundle = keypair.get_bundle(&mut passphrase, "hint".to_string());

        let keypair_from_bundle = Keypair::from_bundle(&bundle, &mut passphrase);

        assert_eq!(64, keypair_from_bundle.sign_priv.len());
        assert_eq!(32, keypair_from_bundle.enc_priv.len());
        assert_eq!(64, keypair_from_bundle.pub_keys.len());
    }

    #[test]
    fn it_should_get_id() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed);

        let mut pk: SecBuf = keypair.get_id();
        let pk = pk.read_lock();
        println!("pk: {:?}", *pk);
        let mut pk1: SecBuf = keypair.get_id();
        let pk1 = pk1.read_lock();
        println!("pk1: {:?}", *pk1);
        assert_eq!(format!("{:?}", *pk), format!("{:?}", *pk1));
    }

    #[test]
    fn it_should_get_bundle() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair = Keypair::new_from_seed(&mut seed);
        let mut passphrase = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut passphrase);

        let bundle: util::Bundle = keypair.get_bundle(&mut passphrase, "hint".to_string());

        // println!("HINT: {:?}",bundle.hint);
        // println!("{:?}",bundle.data.pw_pub_keys.salt);
        assert_eq!("hint", bundle.hint);
    }

    #[test]
    fn it_should_encode_n_decode_data_for_multiple_users2() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair_main = Keypair::new_from_seed(&mut seed);

        let mut seed_1 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_1);
        let mut keypair_1 = Keypair::new_from_seed(&mut seed_1);

        let mut seed_2 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_2);
        let mut keypair_2 = Keypair::new_from_seed(&mut seed_2);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let recipient_id = vec![&mut keypair_1.pub_keys, &mut keypair_2.pub_keys];

        let mut out = Vec::new();
        keypair_main.encrypt(recipient_id, &mut message, &mut out);

        match keypair_2.decrypt(&mut keypair_main.pub_keys, &mut out) {
            Ok(mut dm) => {
                let message = message.read_lock();
                let dm = dm.read_lock();
                assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
            }
            Err(_) => {
                assert!(false);
            }
        };
    }
    #[test]
    fn it_should_encode_n_decode_data_for_multiple_users1() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair_main = Keypair::new_from_seed(&mut seed);

        let mut seed_1 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_1);
        let mut keypair_1 = Keypair::new_from_seed(&mut seed_1);

        let mut seed_2 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_2);
        let mut keypair_2 = Keypair::new_from_seed(&mut seed_2);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let recipient_id = vec![&mut keypair_1.pub_keys, &mut keypair_2.pub_keys];

        let mut out = Vec::new();
        keypair_main.encrypt(recipient_id, &mut message, &mut out);

        match keypair_1.decrypt(&mut keypair_main.pub_keys, &mut out) {
            Ok(mut dm) => {
                println!("Decrypted Message: {:?}", dm);
                let message = message.read_lock();
                let dm = dm.read_lock();
                assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
            }
            Err(_) => {
                println!("Error");
                assert!(false);
            }
        };
    }
    #[test]
    fn it_should_with_fail_when_wrong_key_used_to_decrypt() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair_main = Keypair::new_from_seed(&mut seed);

        let mut seed_1 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_1);
        let mut keypair_1 = Keypair::new_from_seed(&mut seed_1);

        let mut seed_2 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_2);
        let mut keypair_2 = Keypair::new_from_seed(&mut seed_2);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let recipient_id = vec![&mut keypair_1.pub_keys];

        let mut out = Vec::new();
        keypair_main.encrypt(recipient_id, &mut message, &mut out);

        keypair_2
            .decrypt(&mut keypair_main.pub_keys, &mut out)
            .expect_err("should have failed");
    }
    #[test]
    fn it_should_encode_n_decode_data() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);
        let mut keypair_main = Keypair::new_from_seed(&mut seed);

        let mut seed_1 = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed_1);
        let mut keypair_1 = Keypair::new_from_seed(&mut seed_1);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let recipient_id = vec![&mut keypair_1.pub_keys];

        let mut out = Vec::new();
        keypair_main.encrypt(recipient_id, &mut message, &mut out);

        match keypair_1.decrypt(&mut keypair_main.pub_keys, &mut out) {
            Ok(mut dm) => {
                let message = message.read_lock();
                let dm = dm.read_lock();
                assert_eq!(format!("{:?}", *message), format!("{:?}", *dm));
            }
            Err(_) => {
                assert!(false);
            }
        };
    }
    #[test]
    fn it_should_sign_message_and_verify() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);

        let mut keypair = Keypair::new_from_seed(&mut seed);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_secure(64);

        keypair.sign(&mut message, &mut message_signed);

        let check = keypair.verify(&mut message_signed, &mut message);
        assert_eq!(0, check);
    }
    #[test]
    fn it_should_set_keypair_from_seed() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);

        let keypair = Keypair::new_from_seed(&mut seed);

        // let pub_keys = keypair.pub_keys.read_lock();
        // println!("{:?}",pub_keys);
        // let sign_priv = keypair.sign_priv.read_lock();
        // println!("{:?}",sign_priv);
        // let enc_priv = keypair.enc_priv.read_lock();
        // println!("{:?}",enc_priv);

        assert_eq!(64, keypair.pub_keys.len());
        assert_eq!(64, keypair.sign_priv.len());
        assert_eq!(32, keypair.enc_priv.len());
    }
}
