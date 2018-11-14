use cas::content::{Address, AddressableContent, Content};
use entry::{Entry, ToEntry};
use entry_type::EntryType;
use json::JsonString;

use std::convert::TryFrom;

use super::super::error::HolochainError;
use reed_solomon::{Decoder, Encoder};

const PARITY_LEN: usize = 2;

pub type Identity = String;

#[derive(Clone)]
pub struct IdentityBuffer([u8; 64]);

impl IdentityBuffer {
    pub fn parse(s: &str) -> Result<IdentityBuffer, HolochainError> {
        let s = s.replace("-", "+").replace("_", "/");
        let s = base64::decode(&s)?;
        let dec = Decoder::new(PARITY_LEN);
        let dec = *dec.correct(s.as_slice(), None)?;
        Ok(IdentityBuffer(array_ref![dec, 0, 64].clone()))
    }

    pub fn get_sig(&self) -> &[u8; 32] {
        array_ref![self.0, 0, 32]
    }

    pub fn get_enc(&self) -> &[u8; 32] {
        array_ref![self.0, 32, 32]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent(Identity);

impl Agent {
    pub fn generate_fake(s: &str) -> Self {
        let mut s = s.to_string();
        while s.len() < 85 {
            s.push_str("+");
        }
        s.push_str("A==");
        let s = base64::decode(&s)
            .expect("could not decode the generated fake base64 string - use the base64 alphabet");
        let s = IdentityBuffer(array_ref![s, 0, 64].clone());
        Agent::render(&s)
    }

    pub fn render(b: &IdentityBuffer) -> Self {
        let enc = Encoder::new(PARITY_LEN);
        let enc = *enc.encode(&b.0);
        Agent(base64::encode(&enc[..]).replace("+", "-").replace("/", "_"))
    }

    pub fn correct(s: &str) -> Result<Self, HolochainError> {
        Ok(Agent::render(&IdentityBuffer::parse(s)?))
    }

    pub fn to_buffer(&self) -> Result<IdentityBuffer, HolochainError> {
        IdentityBuffer::parse(&self.0)
    }
}

impl<'a> std::convert::TryFrom<&'a String> for Agent {
    type Error = HolochainError;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        Agent::correct(s.as_str())
    }
}

impl<'a> std::convert::TryFrom<&'a str> for Agent {
    type Error = HolochainError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Agent::correct(s)
    }
}

// TODO - conflicts with the bad `From` below
/*
impl std::convert::TryFrom<String> for Agent {
    type Error = HolochainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Agent::correct(&s)
    }
}
*/

// TODO - this isn't good, but we have a lot of things using it
// so... go ahead and implement for now
impl From<String> for Agent {
    fn from(s: String) -> Self {
        match Agent::correct(&s) {
            Ok(a) => a,
            Err(e) => panic!("failed to parse Agent identity: {:?}", e),
        }
    }
}

impl From<Agent> for String {
    fn from(agent: Agent) -> String {
        String::from(agent.0)
    }
}

impl std::convert::TryFrom<JsonString> for Agent {
    type Error = HolochainError;

    fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
        let json_string = String::from(json_string);
        let json_string: serde_json::Value = match serde_json::from_str(&json_string) {
            Ok(d) => d,
            Err(e) => return Err(HolochainError::SerializationError(e.to_string())),
        };
        let json_string = match json_string.as_str() {
            Some(s) => s.to_string(),
            None => return Err(HolochainError::SerializationError("bad identity".into())),
        };
        Agent::correct(&json_string)
    }
}

impl<'a> From<&'a Agent> for JsonString {
    fn from(agent: &Agent) -> JsonString {
        serde_json::Value::String(agent.0.clone()).into()
    }
}

impl From<Agent> for JsonString {
    fn from(agent: Agent) -> JsonString {
        JsonString::from(&agent)
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
        self.0.clone().into()
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
        format!("\"{}\"", GOOD_ID).into()
    }

    pub fn test_agent() -> Agent {
        Agent::correct(BAD_ID).unwrap()
    }

    #[test]
    fn it_should_allow_buffer_access() {
        let buf = test_agent().to_buffer().unwrap();
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
        assert_eq!("sandwich-----------------------------------------------------------------------------BpJ".to_string(), String::from(Agent::generate_fake("sandwich")));
    }

    #[test]
    fn it_should_correct_errors() {
        assert_eq!(GOOD_ID.to_string(), String::from(test_agent()));
    }

    #[test]
    fn it_should_use_self_as_address() {
        assert_eq!(GOOD_ID.to_string(), String::from(test_agent().address()));
    }

    #[test]
    fn it_should_try_from_string_ref() {
        let test: String = Agent::try_from(&BAD_ID.to_string()).unwrap().into();
        assert_eq!(GOOD_ID.to_string(), test);
    }

    #[test]
    fn it_should_try_from_str_ref() {
        let test: String = Agent::try_from(BAD_ID.to_string().as_str()).unwrap().into();
        assert_eq!(GOOD_ID.to_string(), test);
    }

    #[test]
    fn it_should_from_string() {
        let test: String = Agent::from(BAD_ID.to_string()).into();
        assert_eq!(GOOD_ID.to_string(), test);
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
            Content::from("{\"value\":\"\\\"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd\\\"\",\"entry_type\":\"%agent_id\"}");
        // content()
        assert_eq!(expected_content, test_agent().content(),);

        // from_content()
        assert_eq!(test_agent(), Agent::from_content(&expected_content),);
    }
}
