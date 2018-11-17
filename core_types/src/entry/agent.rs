//! Represents an agent entry in the cas

use cas::content::{Address, AddressableContent, Content};
use entry::{Entry, EntryType, ToEntry};
use json::JsonString;

use std::convert::TryFrom;

use super::super::error::HolochainError;
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
pub struct Agent {
    /// a nickname for referencing this agent
    pub nick: String,
    /// the base64url encoded public identity string for this agent
    pub key: String,
}

impl Agent {
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
        Agent::new(s, &buf)
    }

    /// initialize an Agent struct with `nick` and `key`
    pub fn new(nick: &str, key: &KeyBuffer) -> Self {
        Agent {
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

impl ToEntry for Agent {
    /// convert Agent to an entry
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::AgentId, JsonString::from(self))
    }

    /// build an Agent from an entry
    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::AgentId, entry.entry_type());
        match Agent::try_from(entry.value().to_owned()) {
            Ok(a) => a,
            Err(e) => panic!("failed to parse Agent entry: {:?}", e),
        }
    }
}

impl AddressableContent for Agent {
    /// for an Agent, the address is their public base64url encoded itentity string
    fn address(&self) -> Address {
        self.key.clone().into()
    }

    /// get the entry content
    fn content(&self) -> Content {
        self.to_entry().content()
    }

    /// build from entry content
    fn from_content(content: &Content) -> Self {
        Agent::from_entry(&Entry::from_content(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static GOOD_ID: &'static str =
        "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";
    static BAD_ID: &'static str =
        "ATIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";

    pub fn test_identity_value() -> Content {
        format!("{{\"nick\":\"bob\",\"key\":\"{}\"}}", GOOD_ID).into()
    }

    pub fn test_agent() -> Agent {
        Agent::new("bob", &KeyBuffer::with_corrected(BAD_ID).unwrap())
    }

    #[test]
    fn it_should_allow_buffer_access() {
        let buf = test_agent().to_buffer();
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
        assert_eq!("sandwich----------------------------------------------------------------------------AA-k".to_string(), Agent::generate_fake("sandwich").address().to_string());
    }

    #[test]
    fn it_should_correct_errors() {
        assert_eq!(GOOD_ID.to_string(), test_agent().address().to_string());
    }

    #[test]
    /// show ToString implementation for Agent
    fn agent_to_string_test() {
        assert_eq!(test_identity_value(), test_agent().into());
    }

    #[test]
    /// show ToEntry implementation for Agent
    fn agent_to_entry_test() {
        // to_entry()
        assert_eq!(
            Entry::new(EntryType::AgentId, test_identity_value()),
            test_agent().to_entry(),
        );

        // from_entry()
        assert_eq!(
            test_agent(),
            Agent::from_entry(&Entry::new(EntryType::AgentId, test_identity_value())),
        );
    }

    #[test]
    /// show AddressableContent implementation for Agent
    fn agent_addressable_content_test() {
        let expected_content =
            Content::from("{\"value\":\"{\\\"nick\\\":\\\"bob\\\",\\\"key\\\":\\\"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd\\\"}\",\"entry_type\":\"%agent_id\"}");
        // content()
        assert_eq!(expected_content, test_agent().content(),);

        // from_content()
        assert_eq!(test_agent(), Agent::from_content(&expected_content),);
    }
}
