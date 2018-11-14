//! Agent Ids are 64 bytes long.
//! - The first 32 bytes 0-32 are the public side of a libsodium signing keypair
//! - The next 32 bytes 32-64 are the public side of a libsodium kx keypair
//! When displayed to users 2 reed-solomon parity bytes are appended to the end,
//! and the resulting buffer is base64url encoded.
//! (https://tools.ietf.org/html/rfc4648#section-5)

use std::convert::TryFrom;

use super::error::HolochainError;
use reed_solomon::{Decoder, Encoder};

const PARITY_LEN: usize = 2;

/// Represents a public agent identity
pub struct AgentIdPub([u8; 64]);

impl AgentIdPub {
    /// get the signing public key portion of this identity
    pub fn get_sig(&self) -> &[u8; 32] {
        array_ref![self.0, 0, 32]
    }

    /// get the encryption (kx) public key portion of this identity
    pub fn get_enc(&self) -> &[u8; 32] {
        array_ref![self.0, 32, 32]
    }
}

impl std::fmt::Display for AgentIdPub {
    /// display this AgentIdPub as a user string
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl std::fmt::Debug for AgentIdPub {
    /// output the raw bytes of this public identity
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "AgentIdPub(sig({:?}), enc({:?}))",
            self.get_sig(),
            self.get_enc()
        )
    }
}

impl<'a> From<&'a AgentIdPub> for String {
    /// stringify this agent public identity
    fn from(id: &AgentIdPub) -> Self {
        let enc = Encoder::new(PARITY_LEN);
        let enc = *enc.encode(&id.0[0..64]);
        base64::encode(&enc[..]).replace("+", "-").replace("/", "_")
    }
}

impl From<AgentIdPub> for String {
    /// stringify this agent public identity
    fn from(id: AgentIdPub) -> Self {
        String::from(&id)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for AgentIdPub {
    type Error = HolochainError;

    /// import this agent identity from a string (using reed-solomon correction)
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.replace("-", "+").replace("_", "/");
        let s = base64::decode(&s)?;
        let dec = Decoder::new(PARITY_LEN);
        let dec = *dec.correct(s.as_slice(), None)?;
        Ok(AgentIdPub(array_ref![dec, 0, 64].clone()))
    }
}

impl<'a> std::convert::TryFrom<&'a String> for AgentIdPub {
    type Error = HolochainError;

    /// import this agent identity from a string (using reed-solomon correction)
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        AgentIdPub::try_from(s.as_str())
    }
}

impl std::convert::TryFrom<String> for AgentIdPub {
    type Error = HolochainError;

    /// import this agent identity from a string (using reed-solomon correction)
    fn try_from(s: String) -> Result<Self, Self::Error> {
        AgentIdPub::try_from(s.as_str())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    static GOOD_ID: &'static str =
        "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";
    static BAD_ID: &'static str =
        "ATIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";

    #[test]
    fn it_should_decode_corrupt() {
        let id = AgentIdPub::try_from(BAD_ID).unwrap();
        let id = String::from(&id);
        assert_eq!(GOOD_ID, id);
    }

    #[test]
    fn it_should_convert_passed() {
        let id = AgentIdPub::try_from(BAD_ID.to_string()).unwrap();
        let id = String::from(id);
        assert_eq!(GOOD_ID, id);
    }

    #[test]
    fn it_should_convert_string_ref() {
        let id = AgentIdPub::try_from(&BAD_ID.to_string()).unwrap();
        let id = String::from(id);
        assert_eq!(GOOD_ID, id);
    }

    #[test]
    fn it_should_be_accessible() {
        let id = AgentIdPub::try_from(GOOD_ID).unwrap();
        assert_eq!(
            &[
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            id.get_sig()
        );
        assert_eq!(
            &[
                51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51,
                52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52
            ],
            id.get_enc()
        );
    }

    #[test]
    fn it_should_display() {
        let id = AgentIdPub::try_from(BAD_ID).unwrap();
        assert_eq!(GOOD_ID, format!("{}", id));
    }

    #[test]
    fn it_should_debug() {
        let id = AgentIdPub::try_from(BAD_ID).unwrap();
        assert_eq!("AgentIdPub(sig([49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50]), enc([51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52]))".to_string(), format!("{:?}", id));
    }
}
