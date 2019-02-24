use crate::{
    cas::content::{Address, AddressableContent, Content},
    entry::Entry,
    error::HcResult,
    json::JsonString,
};

use crate::error::HolochainError;
use std::{convert::TryFrom, str};

use hcid::*;

pub type Base32 = String;

/// AgentId represents an agent in the Holochain framework.
/// This data struct is meant be stored in the CAS and source-chain.
/// Its key is the public signing key, and is also used as its address.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, DefaultJson)]
pub struct AgentId {
    /// a nickname for referencing this agent
    pub nick: String,
    /// the encoded public signing key of this agent (the magnifier)
    pub pub_sign_key: Base32,
}

impl AgentId {
    /// generate a agent id with fake key
    pub fn generate_fake(nick: &str) -> Self {
        let mut key: [u8; 32] = [0; 32];
        key[0] = 42;
        AgentId::new_with_raw_key(nick, str::from_utf8(&key).unwrap())
            .expect("AgentId fake key generation failed")
    }

    /// initialize an Agent struct with `nick` and `key` that will be encoded
    pub fn new_with_raw_key(nick: &str, key: &str) -> HcResult<Self> {
        let codec = with_hcs0()?;
        let key_b32 = codec.encode(key.as_bytes())?;
        Ok(AgentId::new(nick, key_b32))
    }

    /// initialize an Agent struct with `nick` and a Base32 `key` from HCID
    pub fn new(nick: &str, key_b32: Base32) -> Self {
        AgentId {
            nick: nick.to_string(),
            pub_sign_key: key_b32,
        }
    }

    //    pub fn has_authored(&self, data: &mut SecBuf, signature: &mut SecBuf) -> bool {
    //        utils::verify_sign(self.key_b32, data, signature)
    //            .expect("Failed to verify signature with AgentId. Key might be invalid.");
    //    }
}

impl AddressableContent for AgentId {
    /// for an Agent, the address is their public base32 encoded public signing key string
    fn address(&self) -> Address {
        self.pub_sign_key.clone().into()
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
    let key_b32 = codec
        .encode(s.as_bytes())
        .expect("AgentID key decoding failed. Key was not properly encoded.");
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
