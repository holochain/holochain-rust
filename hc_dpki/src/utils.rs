use crate::SEED_SIZE;
use hcid::*;
use holochain_core_types::{agent::Base32, error::HcResult};
use holochain_sodium::{secbuf::SecBuf, sign};
use std::str;

///
pub(crate) fn decode_pub_key_into_secbuf(
    pub_key_b32: &str,
    codec: &HcidEncoding,
) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key = codec.decode(pub_key_b32)?;
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

///
pub(crate) fn encode_pub_key(pub_key_sec: &mut SecBuf, codec: &HcidEncoding) -> HcResult<Base32> {
    let locker = pub_key_sec.read_lock();
    let pub_buf = &*locker;
    let pub_key = array_ref![pub_buf, 0, SEED_SIZE];
    Ok(codec.encode(pub_key)?)
}

/// verify data that was signed with our private signing key
/// @param {Base32} pub_sign_key_b32 - Public signing key to verify with
/// @param {SecBuf} data buffer to verify
/// @param {SecBuf} signature candidate for that data buffer
/// @return true if verification succeeded
pub fn verify_sign(
    pub_sign_key_b32: Base32,
    data: &mut SecBuf,
    signature: &mut SecBuf,
) -> HcResult<bool> {
    let mut pub_key = decode_pub_key_into_secbuf(
        &pub_sign_key_b32,
        &with_hcs0().expect("HCID failed miserably with_hcs0."),
    )?;
    let res = holochain_sodium::sign::verify(signature, data, &mut pub_key);
    Ok(res == 0)
}
