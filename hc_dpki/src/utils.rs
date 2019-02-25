use crate::SEED_SIZE;
use hcid::*;
use holochain_core_types::{agent::Base32, error::HcResult};
use holochain_sodium::{secbuf::SecBuf, sign};

/// Decode an HCID-encoded key into a SecBuf
/// @param {Base32} pub_key_b32 - Public signing key to decode
/// @param {HcidEncoding} codec - The configured HCID decoder to use
/// @return {SecBuf} Resulting decoded key
pub(crate) fn decode_pub_key(pub_key_b32: Base32, codec: &HcidEncoding) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key = codec.decode(&pub_key_b32)?;
    // convert to SecBuf
    let mut pub_key_sec = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    {
        let mut pub_key_lock = pub_key_sec.write_lock();
        for x in 0..pub_key.len() {
            pub_key_lock[x] = pub_key[x];
        }
    }
    Ok(pub_key_sec)
}

/// Encode with HCID a public key given as a SecBuf
/// @param {SecBuf} pub_key_sec - Public signing key to encode
/// @param {HcidEncoding} codec - The configured HCID encoder to use
/// @return {Base32} Resulting HCID encoded key
pub(crate) fn encode_pub_key(pub_key_sec: &mut SecBuf, codec: &HcidEncoding) -> HcResult<Base32> {
    let locker = pub_key_sec.read_lock();
    let pub_buf = &*locker;
    let pub_key = array_ref![pub_buf, 0, SEED_SIZE];
    Ok(codec.encode(pub_key)?)
}

/// Verify data that was signed
/// @param {Base32} pub_sign_key_b32 - Public signing key to verify with
/// @param {SecBuf} data - Data buffer to verify
/// @param {SecBuf} signature - Candidate signature for that data buffer
/// @return true if verification succeeded
pub fn verify(
    pub_sign_key_b32: Base32,
    data: &mut SecBuf,
    signature: &mut SecBuf,
) -> HcResult<bool> {
    let mut pub_key = decode_pub_key(
        pub_sign_key_b32,
        &with_hcs0().expect("HCID failed miserably with_hcs0."),
    )?;
    let res = holochain_sodium::sign::verify(signature, data, &mut pub_key);
    Ok(res == 0)
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

        assert!(pub_sec_buf.is_same(&mut roundtrip));
    }

    #[test]
    fn it_should_verify() {
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
        let res = verify(pub_key_b32, &mut message, &mut signature);
        assert!(res.unwrap());
    }
}
