use crate::holochain_sodium::{
    secbuf::{
        SecBuf,
    },
    kx::{
        PUBLICKEYBYTES,
        SECRETKEYBYTES,
    },
};
use crate::holochain_sodium::{sign,kx};
use crate::util::{
    encode_id,
};
use crate::holochain_sodium::random::random_secbuf;

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


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;
    #[test]
    fn it_should_set_keypair_from_seed() {
        let mut seed = SecBuf::with_secure(SEEDSIZE);
        random_secbuf(&mut seed);

        let mut keypair = Keypair::new_from_seed(&mut seed);

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
