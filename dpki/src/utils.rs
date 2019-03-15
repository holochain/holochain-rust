use crate::{
    password_encryption::{pw_dec, pw_enc, EncryptedData, PwHashConfig},
    CODEC_HCS0, CONTEXT_SIZE, SEED_SIZE,
};
use hcid::*;
use holochain_core_types::{
    agent::Base32,
    cas::content::Address,
    error::{HcResult, HolochainError},
    signature::{Provenance, Signature},
};
use holochain_sodium::{kdf, secbuf::SecBuf, sign};
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

/// Decode an HCID-encoded key into a SecBuf
/// @param {Base32} pub_key_b32 - Public signing key to decode
/// @param {HcidEncoding} codec - The configured HCID decoder to use
/// @return {SecBuf} Resulting decoded key
pub(crate) fn decode_pub_key(pub_key_b32: Base32, codec: &HcidEncoding) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key = codec.decode(&pub_key_b32)?;
    // convert to SecBuf
    let mut pub_key_sec = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    pub_key_sec.from_array(&pub_key)?;
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

    let mut signature_buf = SecBuf::with_insecure(signature_bytes.len());
    signature_buf
        .write(0, signature_bytes.as_slice())
        .expect("SecBuf must be writeable");

    let mut message_buf = SecBuf::with_insecure_from_string(data);
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
    Ok(holochain_sodium::sign::verify(
        signature,
        data,
        &mut pub_key,
    ))
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
        let mut buf = SecBuf::with_insecure(8);
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
    let mut derived_seed_buf = SecBuf::with_secure(size);
    let mut context = seed_context.to_sec_buf();
    kdf::derive(&mut derived_seed_buf, index, &mut context, &mut src_seed)?;
    Ok(derived_seed_buf)
}

/// returns a random seed buf
pub fn generate_random_seed_buf(size: usize) -> SecBuf {
    let mut seed = SecBuf::with_insecure(size);
    seed.randomize();
    seed
}

/// encrypt and base64 encode a secbuf
pub fn encrypt_with_passphrase_buf(
    data_buf: &mut SecBuf,
    passphrase: &mut SecBuf,
    config: Option<PwHashConfig>,
) -> HcResult<String> {
    // encrypt buffer
    let encrypted_blob = pw_enc(data_buf, passphrase, config)?;
    // Serialize and convert to base64
    let serialized_blob = serde_json::to_string(&encrypted_blob).expect("Failed to serialize Blob");
    Ok(base64::encode(&serialized_blob))
}

/// unencode base64 and decrypt a passphrase encrypted blob
pub fn decrypt_with_passphrase_buf(
    blob: &String,
    passphrase: &mut SecBuf,
    config: Option<PwHashConfig>,

    size: usize,
) -> HcResult<SecBuf> {
    // Decode base64
    let blob_b64 = base64::decode(blob)?;
    // Deserialize
    let blob_json = str::from_utf8(&blob_b64)?;
    let encrypted_blob: EncryptedData = serde_json::from_str(&blob_json)?;
    // Decrypt
    let mut decrypted_data = SecBuf::with_secure(size);
    pw_dec(&encrypted_blob, passphrase, &mut decrypted_data, config)?;
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
mod tests {
    use super::*;
    use crate::SIGNATURE_SIZE;
    use hcid::with_hcs0;
    use holochain_sodium::{secbuf::SecBuf, sign};

    #[test]
    fn it_should_hcid_roundtrip() {
        let mut pub_sec_buf = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        pub_sec_buf.randomize();

        let codec = with_hcs0().expect("HCID failed miserably with_hcs0");
        let pub_key_b32 = encode_pub_key(&mut pub_sec_buf, &codec).unwrap();

        let mut roundtrip = decode_pub_key(pub_key_b32, &codec)
            .expect("Public key decoding failed. Key was not properly encoded.");

        assert!(pub_sec_buf.compare(&mut roundtrip) == 0);
    }

    #[test]
    fn it_should_verify_bufs() {
        let codec = with_hcs0().expect("HCID failed miserably with_hcs0");
        // Create random seed
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        seed.randomize();
        // Create keys
        let mut public_key = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        holochain_sodium::sign::seed_keypair(&mut public_key, &mut secret_key, &mut seed).unwrap();
        let pub_key_b32 = encode_pub_key(&mut public_key, &codec).unwrap();
        // Create signing buffers
        let mut message = SecBuf::with_insecure(42);
        message.randomize();
        let mut signature = SecBuf::with_insecure(SIGNATURE_SIZE);
        holochain_sodium::sign::sign(&mut message, &mut secret_key, &mut signature).unwrap();
        let res = verify_bufs(pub_key_b32, &mut message, &mut signature);
        assert!(res.unwrap());
    }

    #[test]
    fn it_should_round_trip_passphrase_encryption() {
        let data_size = 32;
        let mut random_data = SecBuf::with_insecure(data_size);
        random_data.randomize();

        let mut random_passphrase = SecBuf::with_insecure(10);
        random_passphrase.randomize();

        let encrypted_result =
            encrypt_with_passphrase_buf(&mut random_data, &mut random_passphrase, None);
        assert!(encrypted_result.is_ok());

        let encrypted_data = encrypted_result.unwrap();

        let decrypted_result =
            decrypt_with_passphrase_buf(&encrypted_data, &mut random_passphrase, None, data_size);
        assert!(decrypted_result.is_ok());
        let mut decrypted_data = decrypted_result.unwrap();

        assert_eq!(0, decrypted_data.compare(&mut random_data));

        // totally bogus data will return an error
        let bogus_encrypted_data = "askdfklasjdasldkfjlkasdjflkasdjfasdf".to_string();
        let decrypted_result = decrypt_with_passphrase_buf(
            &bogus_encrypted_data,
            &mut random_passphrase,
            None,
            data_size,
        );
        assert!(decrypted_result.is_err());

        // a bogus passphrase will not decrypt to the correct data
        let mut bogus_passphrase = SecBuf::with_insecure(10);
        bogus_passphrase.randomize();
        let decrypted_result = decrypt_with_passphrase_buf(&encrypted_data, &mut bogus_passphrase, None, data_size);
        assert!(decrypted_result.is_ok());
        let mut decrypted_data = decrypted_result.unwrap();
        assert!(0 != decrypted_data.compare(&mut random_data));
    }
}
