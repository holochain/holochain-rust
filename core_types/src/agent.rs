//! Represents an agent entry in the cas

use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::Entry,
    error::HcResult,
    json::JsonString,
};

use std::{convert::TryFrom, str};

use crate::error::HolochainError;
//use reed_solomon::{Decoder, Encoder};

use hcid::*;

//
///// A raw public key buffer
///// Can extract the signature and encryption portions
///// Can parse a base64url encoded user representation
///// Can render a base64url encoded user representation
//#[derive(Clone)]
//pub struct KeyBuffer([u8; KeyBuffer::KEY_LEN]);
//
//impl KeyBuffer {
//    /// Constants specific to KeyBuffer
//    const PARITY_LEN: usize = 5;
//    const KEY_LEN: usize = 64;
//    const HALF_KEY_LEN: usize = KeyBuffer::KEY_LEN / 2;
//
//    /// take a potentially user-entered base64url encoded user representation
//    /// of an public key identity
//    /// apply reed-solomon parity correction
//    /// returns a raw byte buffer
//    pub fn with_corrected(s: &str) -> Result<KeyBuffer, HolochainError> {
//        let s = s.replace("-", "+").replace("_", "/");
//        let base64 = base64::decode(&s)?;
//        let dec = Decoder::new(KeyBuffer::PARITY_LEN);
//        let dec = *dec.correct(base64.as_slice(), None)?;
//        Ok(KeyBuffer::with_raw(array_ref![dec, 0, KeyBuffer::KEY_LEN]))
//    }
//
//    /// generate a key buffer from raw bytes (no correction)
//    pub fn with_raw(b: &[u8; KeyBuffer::KEY_LEN]) -> KeyBuffer {
//        KeyBuffer(*b)
//    }
//
//    /// generate a key buffer from raw bytes from two parts (no correction)
//    pub fn with_raw_parts(
//        a: &[u8; KeyBuffer::HALF_KEY_LEN],
//        b: &[u8; KeyBuffer::HALF_KEY_LEN],
//    ) -> KeyBuffer {
//        let mut buf: [u8; KeyBuffer::KEY_LEN] = [0; KeyBuffer::KEY_LEN];
//
//        buf[..KeyBuffer::HALF_KEY_LEN].clone_from_slice(&a[..KeyBuffer::HALF_KEY_LEN]);
//
//        buf[KeyBuffer::HALF_KEY_LEN..KeyBuffer::KEY_LEN]
//            .clone_from_slice(&b[..KeyBuffer::HALF_KEY_LEN]);
//
//        KeyBuffer(buf)
//    }
//
//    /// render a base64url encoded user identity with reed-solomon parity bytes
//    pub fn render(&self) -> String {
//        let enc = Encoder::new(KeyBuffer::PARITY_LEN);
//        let enc = *enc.encode(&self.0);
//        base64::encode(&enc[..]).replace("+", "-").replace("/", "_")
//    }
//
//    /// get the signature public key portion of this buffer
//    pub fn get_sig(&self) -> &[u8; KeyBuffer::HALF_KEY_LEN] {
//        array_ref![self.0, 0, KeyBuffer::HALF_KEY_LEN]
//    }
//
//    /// get the encryption public key portion of this buffer
//    pub fn get_enc(&self) -> &[u8; KeyBuffer::HALF_KEY_LEN] {
//        array_ref![self.0, KeyBuffer::HALF_KEY_LEN, KeyBuffer::HALF_KEY_LEN]
//    }
//}

pub type Base32 = String;

/// agent data that can be stored in the CAS / source-chain
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct AgentId {
    /// a nickname for referencing this agent
    pub nick: String,
    /// the encoded public signing key of this agent (the magnifier)
    pub key_b32: Base32,
}

impl AgentId {
    /// generate a agent id with fake key
//    pub fn generate_fake(nick: &str) -> Self {
//        let mut buf = nick.to_string();
//        // Make sure base64 string must is big enough to decode into 64 bytes key
//        while buf.len() < 82 {
//            buf.push_str("+");
//        }
//        buf.push_str("AAAA");
//        let buf = base64::decode(&buf)
//            .expect("could not decode the generated fake base64 string - use the base64 alphabet");
//        let buf = KeyBuffer::with_raw(array_ref![buf, 0, KeyBuffer::KEY_LEN]);
//        AgentId::new(nick, &buf)
//    }

    pub fn generate_fake(nick: &str) -> Self {
        let key = "42";
        AgentId::new_with_key(nick, key).expect("AgentId fake key generation failed.")
    }

    /// initialize an Agent struct with `nick` and `key`
    pub fn new_with_key(nick: &str, key: &str) -> HcResult<Self> {
        let codec = with_hcs0()?;
        let key_b32 = codec.encode(key.as_bytes())?;
        Ok(AgentId::new(nick, key_b32))
    }

    /// initialize an Agent struct with `nick` and `key`
    pub fn new(nick: &str, key_b32: Base32) -> Self {
        AgentId {
            nick: nick.to_string(),
            key_b32,
        }
    }




//    /// get a key buffer based on this agent's key (no correction)
//    pub fn to_buffer(&self) -> KeyBuffer {
//        let s = self.key.replace("-", "+").replace("_", "/");
//        let key = base64::decode(&s).expect("corrupt identity key");
//        KeyBuffer::with_raw(array_ref![key, 0, KeyBuffer::KEY_LEN])
//    }
}

impl AddressableContent for AgentId {
    /// for an Agent, the address is their public base64url encoded itentity string
    fn address(&self) -> Address {
        self.key_b32.clone().into()
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
    "sandwich--------------------------------------------------------------------------AAAEqzh28L";
pub static BAD_ID: &'static str =
    "asndwich--------------------------------------------------------------------------AAAEqzh28L";
pub static TOO_BAD_ID: &'static str =
    "asadwich--------------------------------------------------------------------------AAAEqzh28L";

//pub fn test_base64_to_agent_id(s: &str) -> Result<AgentId, HolochainError> {
//    let key = &KeyBuffer::with_corrected(s)?;
//    Ok(AgentId::new("bob", key))
//}

pub fn test_base32_to_agent_id(s: &str) -> Result<AgentId, HolochainError> {
    let codec = with_hcs0().expect("HCID failed miserably.");
    let key_b32 = codec.encode(s.as_bytes()).expect("AgentID key decoding failed. Key was not properly encoded.");
    Ok(AgentId::new("bob", key_b32))
}

pub fn test_agent_id() -> AgentId {
    test_base32_to_agent_id(BAD_ID).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn test_identity_value() -> Content {
        format!("{{\"nick\":\"bob\",\"key\":\"{}\"}}", GOOD_ID).into()
    }
//
//    #[test]
//    fn it_should_allow_buffer_with_pair() {
//        // let buf = test_base64_to_agent_id(GOOD_ID).unwrap().to_buffer();
//        let buf = KeyBuffer::with_raw_parts(
//            &[
//                177, 169, 221, 194, 39, 33, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239,
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239,
//            ],
//            &[
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190,
//                251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 224, 0, 0,
//            ],
//        );
//        assert_eq!(
//            &[
//                177, 169, 221, 194, 39, 33, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239,
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239
//            ],
//            buf.get_sig()
//        );
//        assert_eq!(
//            &[
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190,
//                251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 224, 0, 0
//            ],
//            buf.get_enc()
//        );
//    }
//
//    #[test]
//    fn it_should_allow_buffer_access() {
//        let buf = test_base64_to_agent_id(GOOD_ID).unwrap().to_buffer();
//
//        assert_eq!(
//            &[
//                177, 169, 221, 194, 39, 33, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239,
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239
//            ],
//            buf.get_sig()
//        );
//        assert_eq!(
//            &[
//                190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190,
//                251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 239, 190, 251, 224, 0, 0
//            ],
//            buf.get_enc()
//        );
//    }

    #[test]
    fn it_can_generate_fake() {
        let agent_id = AgentId::generate_fake("sandwich");
        assert_eq!(
            "sandwich--------------------------------------------------------------------------AAAEqzh28L".to_string(),
            agent_id.address().to_string(),
        );
    }

    #[test]
    fn it_should_correct_errors() {
        assert_eq!(GOOD_ID.to_string(), test_agent_id().address().to_string());
    }

    #[test]
    fn it_fails_if_too_many_errors() {
        let res = test_base32_to_agent_id(TOO_BAD_ID);
        assert!(res.is_err())
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
            Content::from("{\"AgentId\":{\"nick\":\"bob\",\"key\":\"sandwich--------------------------------------------------------------------------AAAEqzh28L\"}}");
        // content()
        assert_eq!(expected_content, test_agent_id().content(),);

        // from_content()
        assert_eq!(
            test_agent_id(),
            AgentId::try_from_content(&expected_content).unwrap(),
        );
    }
}
