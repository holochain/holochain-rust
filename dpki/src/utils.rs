use crate::{
    password_encryption::{pw_dec, pw_enc, EncryptedData},
    SecBuf, CODEC_HCS0, CONTEXT_SIZE, CRYPTO, SEED_SIZE,
};
use hcid::*;
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
    signature::{Provenance, Signature},
};
use holochain_persistence_api::cas::content::Address;
use std::str;

/// a trait for things that have a provenance that can be verified
pub trait Verify {
    fn verify(&self, data: String) -> HcResult<bool>;
}

impl Verify for Provenance {
    fn verify(&self, data: String) -> HcResult<bool> {
        crate::utils::verify(self.source(), data, self.signature())
    }
}

pub fn secbuf_from_array(buf: &mut SecBuf, data: &[u8]) -> HcResult<()> {
    if data.len() != buf.len() {
        return Err(HolochainError::ErrorGeneric(
            "Input does not have same size as SecBuf".to_string(),
        ));
    }
    buf.write(0, data)?;
    Ok(())
}

pub fn secbuf_new_insecure_from_string(data: String) -> SecBuf {
    let u8_data = data.as_bytes();
    let mut buf = CRYPTO.buf_new_insecure(u8_data.len());
    buf.write(0,u8_data).expect("FIXME");
    buf
}

/// Decode an HCID-encoded key into a SecBuf
/// @param {Base32} pub_key_b32 - Public signing key to decode
/// @param {HcidEncoding} codec - The configured HCID decoder to use
/// @return {SecBuf} Resulting decoded key
pub(crate) fn decode_pub_key(pub_key_b32: Base32, codec: &HcidEncoding) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key = codec.decode(&pub_key_b32)?;
    // convert to SecBuf
    let mut pub_key_sec = CRYPTO.buf_new_insecure(CRYPTO.sign_public_key_bytes());
    secbuf_from_array(&mut pub_key_sec, &pub_key)?;
    // Done
    Ok(pub_key_sec)
}

/// Encode with HCID a public key given as a SecBuf
/// @param {SecBuf} pub_key_sec - Public signing key to encode
/// @param {HcidEncoding} codec - The configured HCID encoder to use
/// @return {Base32} Resulting HCID encoded key
pub(crate) fn encode_pub_key(pub_key_sec: &mut SecBuf, codec: &HcidEncoding) -> HcResult<Base32> {
    let locker = pub_key_sec.read_lock();
    Ok(codec.encode(&locker[0..SEED_SIZE])?)
}

/// Verify that an address signed some data
pub fn verify(source: Address, data: String, signature: Signature) -> HcResult<bool> {
    let signature_string: String = signature.into();
    let signature_bytes: Vec<u8> = base64::decode(&signature_string)
        .map_err(|_| HolochainError::ErrorGeneric("Signature syntactically invalid".to_string()))?;

    let mut signature_buf = CRYPTO.buf_new_insecure(signature_bytes.len());
    signature_buf
        .write(0, signature_bytes.as_slice())
        .expect("SecBuf must be writeable");

    let mut message_buf = secbuf_new_insecure_from_string(data);
    verify_bufs(source.to_string(), &mut message_buf, &mut signature_buf)
}

/// Verify data that was signed
/// @param {Base32} pub_sign_key_b32 - Public signing key to verify with
/// @param {SecBuf} data - Data buffer to verify
/// @param {SecBuf} signature - Candidate signature for that data buffer
/// @return true if verification succeeded
pub fn verify_bufs(
    pub_sign_key_b32: Base32,
    data: &mut SecBuf,
    signature: &mut SecBuf,
) -> HcResult<bool> {
    let mut pub_key = decode_pub_key(pub_sign_key_b32, &CODEC_HCS0)?;
    let result = CRYPTO.sign_verify(signature, data, &mut pub_key)?;
    Ok(result)
}

pub struct SeedContext {
    inner: [u8; 8],
}

impl SeedContext {
    pub fn new(data: [u8; 8]) -> Self {
        assert_eq!(data.len(), CONTEXT_SIZE);
        assert!(data.is_ascii());
        SeedContext { inner: data }
    }

    pub fn to_sec_buf(&self) -> SecBuf {
        let mut buf = CRYPTO.buf_new_insecure(8);
        buf.write(0, &self.inner).expect("SecBuf must be writeable");
        buf
    }
}

/// derive a seed from a source seed
pub fn generate_derived_seed_buf(
    mut src_seed: &mut SecBuf,
    seed_context: &SeedContext,
    index: u64,
    size: usize,
) -> HcResult<SecBuf> {
    if index == 0 {
        return Err(HolochainError::ErrorGeneric("Invalid index".to_string()));
    }
    let mut derived_seed_buf = CRYPTO.buf_new_secure(size);
    let mut context = seed_context.to_sec_buf();
    CRYPTO.kdf(&mut derived_seed_buf, index, &mut context, &mut src_seed)?;
    Ok(derived_seed_buf)
}

/// returns a random buf
pub fn generate_random_buf(size: usize) -> SecBuf {
    let mut seed = CRYPTO.buf_new_insecure(size);
    CRYPTO.randombytes_buf(&mut seed).expect("FIXME");
    seed
}

/// returns a random seed buf
pub fn generate_random_seed_buf() -> SecBuf {
    generate_random_buf(SEED_SIZE)
}

/// encrypt and base64 encode a secbuf
pub fn encrypt_with_passphrase_buf(
    data_buf: &mut SecBuf,
    passphrase: &mut SecBuf,
) -> HcResult<String> {
    // encrypt buffer
    let encrypted_blob = pw_enc(data_buf, passphrase)?;
    // Serialize and convert to base64
    let serialized_blob = serde_json::to_string(&encrypted_blob).expect("Failed to serialize Blob");
    Ok(base64::encode(&serialized_blob))
}

/// unencode base64 and decrypt a passphrase encrypted blob
pub fn decrypt_with_passphrase_buf(
    blob: &str,
    passphrase: &mut SecBuf,
    size: usize,
) -> HcResult<SecBuf> {
    // Decode base64
    let blob_b64 = base64::decode(blob)?;
    // Deserialize
    let blob_json = str::from_utf8(&blob_b64)?;
    let encrypted_blob: EncryptedData = serde_json::from_str(&blob_json)?;
    // Decrypt
    let mut decrypted_data = CRYPTO.buf_new_secure(size);
    pw_dec(&encrypted_blob, passphrase, &mut decrypted_data)?;
    // Check size
    if decrypted_data.len() != size {
        return Err(HolochainError::ErrorGeneric(
            "Invalid Blob size".to_string(),
        ));
    }
    // Done
    Ok(decrypted_data)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::SIGNATURE_SIZE;
    use lib3h_crypto_api::CryptoSystem;
    use lib3h_sodium::SodiumCryptoSystem;

    lazy_static! {
        pub static ref TEST_CRYPTO: Box<dyn CryptoSystem> =
            Box::new(SodiumCryptoSystem::new().set_pwhash_interactive());
    }

    #[test]
    fn it_should_hcid_roundtrip() {
        let mut pub_sec_buf = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.sign_public_key_bytes());
        TEST_CRYPTO.randombytes_buf(&mut pub_sec_buf).expect("should work");

        let codec = HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with_hcs0");
        let pub_key_b32 = encode_pub_key(&mut pub_sec_buf, &codec).unwrap();

        let mut roundtrip = decode_pub_key(pub_key_b32, &codec)
            .expect("Public key decoding failed. Key was not properly encoded.");

        assert!(pub_sec_buf.compare(&mut roundtrip) == 0);
    }

    #[test]
    fn it_should_verify_bufs() {
        let codec = HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with_hcs0");
        // Create random seed
        let mut seed = TEST_CRYPTO.buf_new_insecure(SEED_SIZE);
        TEST_CRYPTO.randombytes_buf(&mut seed).expect("should work");
        // Create keys
        let mut public_key = TEST_CRYPTO.buf_new_insecure(TEST_CRYPTO.sign_public_key_bytes());
        let mut secret_key = TEST_CRYPTO.buf_new_secure(TEST_CRYPTO.sign_secret_key_bytes());
        TEST_CRYPTO
            .sign_seed_keypair(&mut seed, &mut public_key, &mut secret_key)
            .unwrap();
        let pub_key_b32 = encode_pub_key(&mut public_key, &codec).unwrap();
        // Create signing buffers
        let mut message = TEST_CRYPTO.buf_new_insecure(42);
        TEST_CRYPTO.randombytes_buf(&mut message).expect("should work");
        let mut signature = TEST_CRYPTO.buf_new_insecure(SIGNATURE_SIZE);
        TEST_CRYPTO
            .sign(&mut signature, &mut message, &mut secret_key)
            .unwrap();
        let res = verify_bufs(pub_key_b32, &mut message, &mut signature);
        assert!(res.unwrap());
    }

    #[test]
    fn it_should_round_trip_passphrase_encryption() {
        let data_size = 32;
        let mut random_data = generate_random_buf(data_size);

        let mut random_passphrase = generate_random_buf(10);

        let encrypted_result =
            encrypt_with_passphrase_buf(&mut random_data, &mut random_passphrase);
        assert!(encrypted_result.is_ok());

        let encrypted_data = encrypted_result.unwrap();

        let decrypted_result =
            decrypt_with_passphrase_buf(&encrypted_data, &mut random_passphrase, data_size);
        assert!(decrypted_result.is_ok());
        let decrypted_data = decrypted_result.unwrap();

        assert_eq!(0, decrypted_data.compare(&mut random_data));

        // totally bogus data will return an error
        let bogus_encrypted_data = "askdfklasjdasldkfjlkasdjflkasdjfasdf".to_string();
        let decrypted_result =
            decrypt_with_passphrase_buf(&bogus_encrypted_data, &mut random_passphrase, data_size);
        assert!(decrypted_result.is_err());

        // a bogus passphrase will not decrypt to the correct data
        let mut bogus_passphrase = generate_random_buf(10);
        let decrypted_result =
            decrypt_with_passphrase_buf(&encrypted_data, &mut bogus_passphrase, data_size);
        assert!(decrypted_result.is_ok());
        let decrypted_data = decrypted_result.unwrap();
        assert!(0 != decrypted_data.compare(&mut random_data));
    }
}
