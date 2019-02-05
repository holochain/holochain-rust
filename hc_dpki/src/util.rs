use crate::bundle;
use holochain_core_types::{agent::KeyBuffer, error::HolochainError};
use holochain_sodium::{aead, kx, pwhash, secbuf::SecBuf};

pub type OpsLimit = u64;
pub type MemLimit = usize;
pub type PwHashAlgo = i8;

pub struct PwHashConfig(pub OpsLimit, pub MemLimit, pub PwHashAlgo);

/// simplify the api for generating a password hash with our set parameters
///
/// @param {SecBuf} pass - the password buffer to hash
///
/// @param {SecBuf} salt - if specified, hash with this salt (otherwise random)
///
/// @param {SecBuf} -  Empty hash buf
pub fn pw_hash(
    password: &mut SecBuf,
    salt: &mut SecBuf,
    hash: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> Result<(), HolochainError> {
    let config = config.unwrap_or(PwHashConfig(
        pwhash::OPSLIMIT_SENSITIVE,
        pwhash::MEMLIMIT_SENSITIVE,
        pwhash::ALG_ARGON2ID13,
    ));
    pwhash::hash(password, config.0, config.1, config.2, salt, hash)?;
    Ok(())
}

/// Helper for encrypting a buffer with a pwhash-ed passphrase
///
/// @param {Buffer} data
///
/// @param {string} passphrase
///
/// @return {bundle::ReturnBundleData} - the encrypted data
pub fn pw_enc(
    data: &mut SecBuf,
    passphrase: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> Result<bundle::ReturnBundleData, HolochainError> {
    let mut secret = SecBuf::with_secure(kx::SESSIONKEYBYTES);
    let mut salt = SecBuf::with_insecure(pwhash::SALTBYTES);
    holochain_sodium::random::random_secbuf(&mut salt);
    let mut nonce = SecBuf::with_insecure(aead::NONCEBYTES);
    holochain_sodium::random::random_secbuf(&mut nonce);
    let mut cipher = SecBuf::with_insecure(data.len() + aead::ABYTES);
    pw_hash(passphrase, &mut salt, &mut secret, config)?;
    aead::enc(data, &mut secret, None, &mut nonce, &mut cipher)?;

    let salt = salt.read_lock().to_vec();
    let nonce = nonce.read_lock().to_vec();
    let cipher = cipher.read_lock().to_vec();
    let data = bundle::ReturnBundleData {
        salt,
        nonce,
        cipher,
    };
    Ok(data)
}

/// Helper for decrypting a buffer with a pwhash-ed passphrase
///
/// @param {Buffer} data
///
/// @param {string} passphrase
///
/// @param {SecBuf} - the decrypted data
pub fn pw_dec(
    bundle: &bundle::ReturnBundleData,
    passphrase: &mut SecBuf,
    decrypted_data: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> Result<(), HolochainError> {
    let mut secret = SecBuf::with_secure(kx::SESSIONKEYBYTES);
    let mut salt = SecBuf::with_insecure(pwhash::SALTBYTES);
    convert_vec_to_secbuf(&bundle.salt, &mut salt);
    let mut nonce = SecBuf::with_insecure(bundle.nonce.len());
    convert_vec_to_secbuf(&bundle.nonce, &mut nonce);
    let mut cipher = SecBuf::with_insecure(bundle.cipher.len());
    convert_vec_to_secbuf(&bundle.cipher, &mut cipher);
    pw_hash(passphrase, &mut salt, &mut secret, config)?;
    aead::dec(decrypted_data, &mut secret, None, &mut nonce, &mut cipher)?;
    Ok(())
}

/// Load the Vec<u8> into the SecBuf
pub fn convert_vec_to_secbuf(data: &Vec<u8>, buf: &mut SecBuf) {
    let mut buf = buf.write_lock();
    for x in 0..data.len() {
        buf[x] = data[x];
    }
}

/// Load the [u8] into the SecBuf
pub fn convert_array_to_secbuf(data: &[u8], buf: &mut SecBuf) {
    let mut buf = buf.write_lock();
    for x in 0..data.len() {
        buf[x] = data[x];
    }
}

/// Generate an identity string with a pair of public keys
///
/// @param {SecBuf} signPub - singing public key
///
/// @param {SecBuf} encPub - encryption public key
///
/// @param {SecBuf} id
pub fn encode_id(sign_pub: &mut SecBuf, enc_pub: &mut SecBuf) -> String {
    let sign_pub = sign_pub.read_lock();
    let enc_pub = enc_pub.read_lock();
    let sp = &*sign_pub;
    let ep = &*enc_pub;
    KeyBuffer::with_raw_parts(array_ref![sp, 0, 32], array_ref![ep, 0, 32]).render()
}

/// break an identity string up into a pair of public keys
///
/// @param {string} id
///
/// @param {SecBuf} signPub - Empty singing public key
///
/// @param {SecBuf} encPub - Empty encryption public key
pub fn decode_id(
    key: String,
    sign_pub: &mut SecBuf,
    enc_pub: &mut SecBuf,
) -> Result<(), HolochainError> {
    let id = &KeyBuffer::with_corrected(&key)?;

    let mut sign_pub = sign_pub.write_lock();
    let mut enc_pub = enc_pub.write_lock();

    let sig = id.get_sig();
    let enc = id.get_enc();

    for x in 0..sign_pub.len() {
        sign_pub[x] = sig[x];
    }
    for x in 0..enc_pub.len() {
        enc_pub[x] = enc[x];
    }
    Ok(())
}

/// Check if the buffer is empty i.e. [0,0,0,0,0,0,0,0]
pub fn check_if_wrong_secbuf(buf: &mut SecBuf) -> bool {
    let buf = buf.read_lock();
    println!("Buf{:?}", *buf);
    for i in 0..buf.len() {
        if buf[i] != 0 {
            return true;
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holochain_sodium::random::random_secbuf;

    const TEST_CONFIG: Option<PwHashConfig> = Some(PwHashConfig(
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
        pwhash::ALG_ARGON2ID13,
    ));

    #[test]
    fn it_should_encrypt_data() {
        let mut data = SecBuf::with_insecure(32);
        {
            let mut data = data.write_lock();
            data[0] = 88;
            data[1] = 101;
        }
        let mut password = SecBuf::with_insecure(pwhash::HASHBYTES);
        {
            let mut password = password.write_lock();
            password[0] = 42;
            password[1] = 222;
        }
        let mut bundle: bundle::ReturnBundleData =
            pw_enc(&mut data, &mut password, TEST_CONFIG).unwrap();

        let mut dec_mess = SecBuf::with_insecure(32);
        pw_dec(&mut bundle, &mut password, &mut dec_mess, TEST_CONFIG).unwrap();

        let data = data.read_lock();
        let dec_mess = dec_mess.read_lock();
        assert_eq!(format!("{:?}", *dec_mess), format!("{:?}", *data));
    }

    #[test]
    fn it_should_generate_pw_hash_with_salt() {
        let mut password = SecBuf::with_insecure(pwhash::HASHBYTES);
        let mut pw2_hash = SecBuf::with_insecure(pwhash::HASHBYTES);
        {
            let mut password = password.write_lock();
            password[0] = 42;
            password[1] = 222;
        }
        let mut salt = SecBuf::with_insecure(pwhash::SALTBYTES);
        pw_hash(&mut password, &mut salt, &mut pw2_hash, TEST_CONFIG).unwrap();
        let pw2_hash = pw2_hash.read_lock();
        assert_eq!("[134, 156, 170, 171, 184, 19, 40, 158, 64, 227, 105, 252, 59, 175, 119, 226, 77, 238, 49, 61, 27, 174, 47, 246, 179, 168, 88, 200, 65, 11, 14, 159]",  format!("{:?}", *pw2_hash));
    }

    #[test]
    fn it_should_decode_to_create_pub_key() {
        let mut sign_pub = SecBuf::with_insecure(32);
        random_secbuf(&mut sign_pub);

        let mut enc_pub = SecBuf::with_insecure(32);
        random_secbuf(&mut enc_pub);

        let enc: String = encode_id(&mut sign_pub, &mut enc_pub);

        let mut sign_pub_dec = SecBuf::with_insecure(32);
        let mut enc_pub_dec = SecBuf::with_insecure(32);

        decode_id(enc, &mut sign_pub_dec, &mut enc_pub_dec).unwrap();

        let sign_pub = sign_pub.read_lock();
        let enc_pub = enc_pub.read_lock();
        let sign_pub_dec = sign_pub_dec.read_lock();
        let enc_pub_dec = enc_pub_dec.read_lock();
        assert_eq!(format!("{:?}", *sign_pub), format!("{:?}", *sign_pub_dec));
        assert_eq!(format!("{:?}", *enc_pub), format!("{:?}", *enc_pub_dec));
    }
}
