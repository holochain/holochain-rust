use crate::holochain_sodium::secbuf::SecBuf;
use crate::holochain_sodium::random::random_secbuf;
use crate::holochain_sodium::{
    sign,
    kx,
    aead,
};
use crate::util::{
    encode_id,
    decode_id,
};

pub const SEEDSIZE:usize = 32 as usize;

pub struct Keypair {
    pub_keys:SecBuf,
    sign_priv:SecBuf,
    enc_priv:SecBuf,
}

impl Keypair {
    /**
     * derive the pairs from a 32 byte seed buffer
     * @param {SecBuf} seed - the seed buffer
     */
    pub fn new_from_seed(seed: &mut SecBuf)-> Self {
        let mut seed = seed;
        let mut sign_public_key = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut sign_secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        let mut enc_public_key = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        let mut enc_secret_key = SecBuf::with_secure(kx::SECRETKEYBYTES);

        sign::seed_keypair(&mut sign_public_key, &mut sign_secret_key,&mut seed).unwrap();
        kx::seed_keypair(&mut seed, &mut enc_public_key, &mut enc_secret_key).unwrap();

        let mut pub_id = SecBuf::with_secure(sign::PUBLICKEYBYTES + kx::PUBLICKEYBYTES);
        encode_id(&mut sign_public_key, &mut enc_public_key, &mut pub_id);

        Keypair {
            pub_keys: pub_id,
            sign_priv: sign_secret_key,
            enc_priv:enc_secret_key
        }
    }

    /**
     * sign some arbitrary data with the signing private key
     * @param {Buffer} data - the data to sign
     */
    pub fn sign (&mut self,data:&mut SecBuf,signature:&mut SecBuf) {
      let mut data = data;
      let mut signature = signature;
      let mut sign_priv =&mut self.sign_priv;
      sign::sign(&mut data,&mut sign_priv,&mut signature).unwrap();
    }

    /**
     * verify data that was signed with our private signing key
     * @param {Buffer} signature
     * @param {Buffer} data
     */
    pub fn verify (&mut self,signature:&mut SecBuf, data:&mut SecBuf)->i32 {
        let mut data = data;
        let mut signature = signature;
        let mut pub_keys =&mut self.pub_keys;
        let mut sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);

        decode_id(&mut pub_keys,&mut sign_pub,&mut enc_pub);

        sign::verify(&mut signature,&mut data,&mut sign_pub)
    }

    /**
     * encrypt arbitrary data to be readale by potentially multiple recipients
     * @param {array<string>} recipientIds - multiple recipient identifier strings
     * @param {Buffer} data - the data to encrypt
     * @return {Buffer}
     */
    pub fn encrypt(&mut self,recipient_id:Vec<&mut SecBuf>, data:&mut SecBuf, out:&mut Vec<SecBuf>){
        let mut sym_secret = SecBuf::with_secure(32);
        random_secbuf(&mut sym_secret);

        let mut srv_rx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        let mut srv_tx = SecBuf::with_secure(kx::SESSIONKEYBYTES);

        let mut pub_keys = &mut self.pub_keys;
        let mut sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        decode_id(&mut pub_keys,&mut sign_pub,&mut enc_pub);

        let mut enc_priv = &mut self.enc_priv;

        for client_pk in recipient_id{
            let mut r_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
            let mut r_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
            let mut client_pk = client_pk;

            decode_id(&mut client_pk,&mut r_sign_pub,&mut r_enc_pub);

            kx::server_session(&mut enc_pub,&mut enc_priv,&mut r_enc_pub,&mut srv_rx,&mut srv_tx);

            let mut nonce = SecBuf::with_insecure(16);
            random_secbuf(&mut nonce);
            let mut cipher = SecBuf::with_insecure(sym_secret.len() + aead::ABYTES);

            aead::enc(&mut sym_secret,&mut srv_tx,None,&mut nonce,&mut cipher).unwrap();
            out.push(nonce);
            out.push(cipher);
        }

        let mut nonce = SecBuf::with_insecure(16);
        random_secbuf(&mut nonce);
        let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
        let mut data = data;
        aead::enc(&mut data,&mut sym_secret,None,&mut nonce,&mut cipher).unwrap();
        out.push(nonce);
        out.push(cipher);
    }

    /**
     * attempt to decrypt the cipher buffer (assuming it was targeting us)
     * @param {string} sourceId - identifier string of who encrypted this data
     * @param {Buffer} cipher - the encrypted data
     * @return {Buffer} - the decrypted data
     */
    pub fn decrypt (&mut self,source_id: &mut SecBuf, cipher_bundle: &Vec<SecBuf>){
        let c_b_iter = cipher_bundle.to_owned();
        let mut source_id = source_id;
        let mut source_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut source_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        decode_id(&mut source_id,&mut source_sign_pub,&mut source_enc_pub);

        let mut client_pub_keys = &mut self.pub_keys;
        let mut client_sign_pub = SecBuf::with_secure(sign::PUBLICKEYBYTES);
        let mut client_enc_pub = SecBuf::with_secure(kx::PUBLICKEYBYTES);
        decode_id(&mut client_pub_keys,&mut client_sign_pub,&mut client_enc_pub);
        let mut client_enc_priv = &mut self.enc_priv;

        let mut cli_rx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        let mut cli_tx = SecBuf::with_secure(kx::SESSIONKEYBYTES);
        kx::client_session(&mut client_enc_pub, &mut client_enc_priv, &mut source_enc_pub, &mut cli_rx,&mut cli_tx).unwrap();

        for i in 0..cipher_bundle.len()-4{
            if i%2 == 0 {
                let mut n = &mut c_b_iter[i];
                let mut c = &mut cipher_bundle[i + 1];
                // let mut c = SecBuf::with_secure(kx::SESSIONKEYBYTES);

                let mut decrypted_message = SecBuf::with_insecure(c.len() - aead::ABYTES);

                aead::dec(&mut decrypted_message,&mut cli_rx,None,&mut n,&mut c).unwrap();

                println!("{:?}",decrypted_message );
            }
        }

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;
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
        keypair_main.encrypt(recipient_id,&mut message,&mut out);

        // for o in out {
        //     println!("->{:?}",o);
        // }

        keypair_1.decrypt(&mut keypair_main.pub_keys,&mut out);

        assert!(false);
    }
    #[test]
    fn it_should_sign_message_and_verify() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);

        let mut keypair = Keypair::new_from_seed(&mut seed);

        let mut message = SecBuf::with_secure(16);
        random_secbuf(&mut message);

        let mut message_signed = SecBuf::with_secure(64);

        keypair.sign(&mut message,&mut message_signed);

        let check = keypair.verify(&mut message_signed,&mut message);
        assert_eq!(0,check);
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

        assert_eq!(64,keypair.pub_keys.len());
        assert_eq!(64,keypair.sign_priv.len());
        assert_eq!(32,keypair.enc_priv.len());
    }
}
