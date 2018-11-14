use std::convert::TryFrom;

use super::error::HolochainError;
use reed_solomon::{Decoder, Encoder};

const PARITY_LEN: usize = 2;

pub struct AgentIdPub([u8; 64]);

impl AgentIdPub {
    pub fn get_sig(&self) -> &[u8; 32] {
        array_ref![self.0, 0, 32]
    }

    pub fn get_enc(&self) -> &[u8; 32] {
        array_ref![self.0, 32, 32]
    }
}

impl std::fmt::Display for AgentIdPub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl std::fmt::Debug for AgentIdPub {
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
    fn from(id: &AgentIdPub) -> Self {
        let enc = Encoder::new(PARITY_LEN);
        let enc = *enc.encode(&id.0[0..64]);
        base64::encode(&enc[..])
    }
}

impl From<AgentIdPub> for String {
    fn from(id: AgentIdPub) -> Self {
        String::from(&id)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for AgentIdPub {
    type Error = HolochainError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let dec = Decoder::new(PARITY_LEN);
        let dec = *dec.correct(base64::decode(s)?.as_slice(), None)?;
        Ok(AgentIdPub(array_ref![dec, 0, 64].clone()))
    }
}

impl<'a> std::convert::TryFrom<&'a String> for AgentIdPub {
    type Error = HolochainError;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        AgentIdPub::try_from(s.as_str())
    }
}

impl std::convert::TryFrom<String> for AgentIdPub {
    type Error = HolochainError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        AgentIdPub::try_from(s.as_str())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    static GOOD_ID: &'static str = "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";
    static BAD_ID: &'static str = "ATIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNBkd";

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
        assert_eq!(&[49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50], id.get_sig());
        assert_eq!(&[51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52], id.get_enc());
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
