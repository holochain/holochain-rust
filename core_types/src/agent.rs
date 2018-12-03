//! Represents an agent entry in the cas

use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::Entry,
    error::HcResult,
    json::JsonString,
};

use std::convert::TryFrom;

use crate::error::HolochainError;
use reed_solomon::{Decoder, Encoder};

const PARITY_LEN: usize = 2;

/// A raw public key buffer
/// Can extract the signature and encryption portions
/// Can parse a base64url encoded user representation
/// Can render a base64url encoded user representation
#[derive(Clone)]
pub struct KeyBuffer([u8; 64]);

impl KeyBuffer {
    /// take a potentially user-entered base64url encoded user representation
    /// of an public key identity
    /// apply reed-solomon parity correction
    /// returns a raw byte buffer
    pub fn with_corrected(s: &str) -> Result<KeyBuffer, HolochainError> {
        let s = s.replace("-", "+").replace("_", "/");
        let s = base64::decode(&s)?;
        let dec = Decoder::new(PARITY_LEN);
        let dec = *dec.correct(s.as_slice(), None)?;
        Ok(KeyBuffer::with_raw(array_ref![dec, 0, 64]))
    }

    /// generate a key buffer from raw bytes (no correction)
    pub fn with_raw(b: &[u8; 64]) -> KeyBuffer {
        KeyBuffer(b.clone())
    }

    /// render a base64url encoded user identity with reed-solomon parity bytes
    pub fn render(&self) -> String {
        let enc = Encoder::new(PARITY_LEN);
        let enc = *enc.encode(&self.0);
        base64::encode(&enc[..]).replace("+", "-").replace("/", "_")
    }

    /// get the signature public key portion of this buffer
    pub fn get_sig(&self) -> &[u8; 32] {
        array_ref![self.0, 0, 32]
    }

    /// get the encryption public key portion of this buffer
    pub fn get_enc(&self) -> &[u8; 32] {
        array_ref![self.0, 32, 32]
    }
}

/// agent data that can be stored in the cas
/// note thate the "address" of an agent entry is the base64url encoded
/// public key identity string
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct AgentId {
    /// a nickname for referencing this agent
    pub nick: String,
    /// the base64url encoded public identity string for this agent
    pub key: String,
}

impl AgentId {
    /// generate a fake testing agent
    /// `s` will be used for the `nick` and included in the key string as well
    /// this agent is not cryptographically generated...
    /// it will not be able to sign / encrypt anything
    pub fn generate_fake(s: &str) -> Self {
        let mut buf = s.to_string();
        while buf.len() < 84 {
            buf.push_str("+");
        }
        buf.push_str("AAAA");
        let buf = base64::decode(&buf)
            .expect("could not decode the generated fake base64 string - use the base64 alphabet");
        let buf = KeyBuffer::with_raw(array_ref![buf, 0, 64]);
        AgentId::new(s, &buf)
    }

    /// initialize an Agent struct with `nick` and `key`
    pub fn new(nick: &str, key: &KeyBuffer) -> Self {
        AgentId {
            nick: nick.to_string(),
            key: key.render(),
        }
    }

    /// get a key buffer based on this agent's key (no correction)
    pub fn to_buffer(&self) -> KeyBuffer {
        let key = base64::decode(&self.key).expect("corrupt identity key");
        KeyBuffer::with_raw(array_ref![key, 0, 64])
    }
}

impl AddressableContent for AgentId {
    /// for an Agent, the address is their public base64url encoded itentity string
    fn address(&self) -> Address {
        self.key.clone().into()
    }

    /// get the entry content
    fn content(&self) -> Content {
        Entry::AgentId(self.to_owned()).into()
    }

    // build from entry content
    fn try_from_content(content: &Content) -> HcResult<Self> {
        match Entry::try_from(content)? {
            Entry::AgentId(agent_id) => Ok(agent_id),
            _ => Err(HolochainError::SerializationError(
                "Attempted to load AgentId from non AgentID entry".into(),
            )),
        }
    }
}

pub static GOOD_ID: &'static str =
    "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";
pub static BAD_ID: &'static str =
    "ATIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";

pub fn test_agent_id() -> AgentId {
    AgentId::new("bob", &KeyBuffer::with_corrected(BAD_ID).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn test_identity_value() -> Content {
        format!("{{\"nick\":\"bob\",\"key\":\"{}\"}}", GOOD_ID).into()
    }

    #[test]
    fn it_should_allow_buffer_access() {
        let buf = test_agent_id().to_buffer();
        assert_eq!(
            &[
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            buf.get_sig()
        );
        assert_eq!(
            &[
                51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51,
                52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52
            ],
            buf.get_enc()
        );
    }

    #[test]
    fn it_can_generate_fake() {
        assert_eq!("sandwich----------------------------------------------------------------------------AA-k".to_string(), AgentId::generate_fake("sandwich").address().to_string());
    }

    #[test]
    fn it_should_correct_errors() {
        assert_eq!(GOOD_ID.to_string(), test_agent_id().address().to_string());
    }

    #[test]
    /// show ToString implementation for Agent
    fn agent_to_string_test() {
        assert_eq!(test_identity_value(), test_agent_id().into());
    }

    #[test]
    /// show AddressableContent implementation for Agent
    fn agent_addressable_content_test() {
        let expected_content =
            Content::from("{\"AgentId\":{\"nick\":\"bob\",\"key\":\"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd\"}}");
        // content()
        assert_eq!(expected_content, test_agent_id().content(),);

        // from_content()
        assert_eq!(
            test_agent_id(),
            AgentId::try_from_content(&expected_content).unwrap(),
        );
    }
}
