//! copied from holochain_rust::dpki::utils for sim2h server

use crate::{
    error::{Sim2hError, Sim2hResult},
    wire_message::WireMessage,
    NEW_RELIC_LICENSE_KEY
};
use hcid::*;
pub use holochain_core_types::signature::Provenance;
use holochain_core_types::{
    agent::Base32,
    error::{HcResult, HolochainError},
};
use lib3h_protocol::{data_types::Opaque, types::AgentPubKey};
use lib3h_sodium::{secbuf::SecBuf, sign};

use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedWireMessage {
    pub provenance: Provenance,
    pub payload: Opaque,
}

#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
impl SignedWireMessage {
    pub fn new(message: WireMessage, provenance: Provenance) -> Self {
        SignedWireMessage {
            provenance,
            payload: message.into(),
        }
    }

    pub fn new_with_key(
        mut secret_key: &mut SecBuf,
        agent_id: AgentPubKey,
        message: WireMessage,
    ) -> Sim2hResult<Self> {
        let payload: Opaque = message.into();
        let mut message_buf = SecBuf::with_insecure(payload.len());
        message_buf
            .write(0, &payload)
            .expect("SecBuf must be writeable");

        let mut signature_buf = SecBuf::with_insecure(SIGNATURE_SIZE);
        lib3h_sodium::sign::sign(&mut message_buf, &mut secret_key, &mut signature_buf).unwrap();

        let reader = signature_buf.read_lock();
        let signature = base64::encode(&**reader).into();
        let provenance = Provenance::new(agent_id.into(), signature);
        Ok(SignedWireMessage {
            provenance,
            payload,
        })
    }

    pub fn verify(&self) -> HcResult<bool> {
        let signature_string: String = self.provenance.signature().into();
        let signature_bytes: Vec<u8> = base64::decode(&signature_string).map_err(|_| {
            HolochainError::ErrorGeneric("Signature syntactically invalid".to_string())
        })?;
        let mut signature_buf = SecBuf::with_insecure(signature_bytes.len());
        signature_buf
            .write(0, signature_bytes.as_slice())
            .expect("SecBuf must be writeable");

        let mut message_buf = SecBuf::with_insecure(self.payload.len());
        message_buf
            .write(0, &self.payload)
            .expect("SecBuf must be writeable");
        verify_bufs(
            self.provenance.source().to_string(),
            &mut message_buf,
            &mut signature_buf,
        )
    }
}

impl From<SignedWireMessage> for Opaque {
    fn from(message: SignedWireMessage) -> Opaque {
        serde_json::to_string(&message)
            .expect("wiremessage should serialize")
            .into()
    }
}

impl TryFrom<Opaque> for SignedWireMessage {
    type Error = Sim2hError;
    fn try_from(message: Opaque) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&String::from_utf8_lossy(&message))
            .map_err(|e| format!("{:?}", e))?)
    }
}

pub const SEED_SIZE: usize = 32;
#[allow(dead_code)]
pub(crate) const SIGNATURE_SIZE: usize = 64;

lazy_static! {
    pub static ref CODEC_HCS0: hcid::HcidEncoding =
        hcid::HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with hcs0.");
    pub static ref CODEC_HCK0: hcid::HcidEncoding =
        hcid::HcidEncoding::with_kind("hck0").expect("HCID failed miserably with_hck0.");
}

/// Decode an HCID-encoded key into a SecBuf
/// @param {Base32} pub_key_b32 - Public signing key to decode
/// @param {HcidEncoding} codec - The configured HCID decoder to use
/// @return {SecBuf} Resulting decoded key
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
pub(crate) fn decode_pub_key(pub_key_b32: Base32, codec: &HcidEncoding) -> HcResult<SecBuf> {
    // Decode Base32 public key
    let pub_key = codec.decode(&pub_key_b32)?;
    // convert to SecBuf
    let mut pub_key_sec = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
    pub_key_sec
        .from_array(&pub_key)
        .map_err(|e| HolochainError::new(&format!("{:?}", e)))?;
    // Done
    Ok(pub_key_sec)
}

/// Encode with HCID a public key given as a SecBuf
/// @param {SecBuf} pub_key_sec - Public signing key to encode
/// @param {HcidEncoding} codec - The configured HCID encoder to use
/// @return {Base32} Resulting HCID encoded key
#[allow(dead_code)] //used in test only
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
pub(crate) fn encode_pub_key(pub_key_sec: &mut SecBuf, codec: &HcidEncoding) -> HcResult<Base32> {
    let locker = pub_key_sec.read_lock();
    Ok(codec.encode(&locker[0..SEED_SIZE])?)
}

/// Verify data that was signed
/// @param {Base32} pub_sign_key_b32 - Public signing key to verify with
/// @param {SecBuf} data - Data buffer to verify
/// @param {SecBuf} signature - Candidate signature for that data buffer
/// @return true if verification succeeded
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
pub fn verify_bufs(
    pub_sign_key_b32: Base32,
    data: &mut SecBuf,
    signature: &mut SecBuf,
) -> HcResult<bool> {
    let mut pub_key = decode_pub_key(pub_sign_key_b32, &CODEC_HCS0)?;
    Ok(lib3h_sodium::sign::verify(signature, data, &mut pub_key))
}

/// returns a random buf
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
pub fn generate_random_buf(size: usize) -> SecBuf {
    let mut seed = SecBuf::with_insecure(size);
    seed.randomize();
    seed
}

/// returns a random seed buf
#[holochain_tracing_macros::newrelic_autotrace(SIM2H)]
pub fn generate_random_seed_buf() -> SecBuf {
    generate_random_buf(SEED_SIZE)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use lib3h_sodium::{secbuf::SecBuf, sign};
    #[test]
    fn it_should_hcid_roundtrip() {
        let mut pub_sec_buf = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        pub_sec_buf.randomize();

        let codec = HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with_hcs0");
        let pub_key_b32 = encode_pub_key(&mut pub_sec_buf, &codec).unwrap();

        let mut roundtrip = decode_pub_key(pub_key_b32, &codec)
            .expect("Public key decoding failed. Key was not properly encoded.");

        assert!(pub_sec_buf.compare(&mut roundtrip) == 0);
    }

    pub fn make_test_agent_with_private_key(seed: &str) -> (AgentPubKey, SecBuf) {
        let codec = HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with_hcs0");
        let mut seed_buf = SecBuf::with_insecure(SEED_SIZE);
        seed_buf
            .write(0, seed.as_bytes())
            .expect("SecBuf must be writeable");
        let mut public_key = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        lib3h_sodium::sign::seed_keypair(&mut public_key, &mut secret_key, &mut seed_buf).unwrap();
        let pub_key_b32 = encode_pub_key(&mut public_key, &codec).unwrap();
        (pub_key_b32.into(), secret_key)
    }

    #[test]
    fn it_should_verify_signed_wire_message() {
        let (agent_id, mut secret_key) = make_test_agent_with_private_key("test_agent");

        let message = WireMessage::Err("fake_error".into());

        let signed_message = SignedWireMessage::new_with_key(&mut secret_key, agent_id, message)
            .expect("should construct");
        assert_eq!(Ok(true), signed_message.verify());
    }

    #[test]
    fn it_should_verify_bufs() {
        let codec = HcidEncoding::with_kind("hcs0").expect("HCID failed miserably with_hcs0");
        // Create random seed
        let mut seed = SecBuf::with_insecure(SEED_SIZE);
        seed.randomize();
        // Create keys
        let mut public_key = SecBuf::with_insecure(sign::PUBLICKEYBYTES);
        let mut secret_key = SecBuf::with_secure(sign::SECRETKEYBYTES);
        lib3h_sodium::sign::seed_keypair(&mut public_key, &mut secret_key, &mut seed).unwrap();
        let pub_key_b32 = encode_pub_key(&mut public_key, &codec).unwrap();
        // Create signing buffers
        let mut message = SecBuf::with_insecure(42);
        message.randomize();
        let mut signature = SecBuf::with_insecure(SIGNATURE_SIZE);
        lib3h_sodium::sign::sign(&mut message, &mut secret_key, &mut signature).unwrap();
        let res = verify_bufs(pub_key_b32, &mut message, &mut signature);
        assert!(res.unwrap());
    }
}
