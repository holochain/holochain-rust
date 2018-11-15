use cas::content::{Address, AddressableContent, Content};
use entry::{Entry, ToEntry};
use entry_type::EntryType;
use json::JsonString;

use std::convert::TryFrom;

use super::super::error::HolochainError;
use reed_solomon::{Decoder, Encoder};

const PARITY_LEN: usize = 2;

#[derive(Clone)]
pub struct KeyBuffer([u8; 64]);

impl KeyBuffer {
    pub fn with_corrected(s: &str) -> Result<KeyBuffer, HolochainError> {
        let s = s.replace("-", "+").replace("_", "/");
        let s = base64::decode(&s)?;
        let dec = Decoder::new(PARITY_LEN);
        let dec = *dec.correct(s.as_slice(), None)?;
        Ok(KeyBuffer::with_raw(array_ref![dec, 0, 64]))
    }

    pub fn with_raw(b: &[u8; 64]) -> KeyBuffer {
        KeyBuffer(b.clone())
    }

    pub fn render(&self) -> String {
        let enc = Encoder::new(PARITY_LEN);
        let enc = *enc.encode(&self.0);
        base64::encode(&enc[..]).replace("+", "-").replace("/", "_")
    }

    pub fn get_sig(&self) -> &[u8; 32] {
        array_ref![self.0, 0, 32]
    }

    pub fn get_enc(&self) -> &[u8; 32] {
        array_ref![self.0, 32, 32]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct Agent {
    pub nick: String,
    pub key: String,
}

impl Agent {
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

    pub fn new(nick: &str, key: &KeyBuffer) -> Self {
        Agent {
            nick: nick.to_string(),
            key: key.render(),
        }
    }

    pub fn to_buffer(&self) -> KeyBuffer {
        let key = base64::decode(&self.key).expect("corrupt identity key");
        KeyBuffer::with_raw(array_ref![key, 0, 64])
    }
}

impl ToEntry for Agent {
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::AgentId, JsonString::from(self))
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::AgentId, entry.entry_type());
        match Agent::try_from(entry.value().to_owned()) {
            Ok(a) => a,
            Err(e) => panic!("failed to parse Agent entry: {:?}", e),
        }
    }
}

impl AddressableContent for Agent {
    fn address(&self) -> Address {
        self.key.clone().into()
    }

    fn content(&self) -> Content {
        self.to_entry().content()
    }

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
