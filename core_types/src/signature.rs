//! The Signature type is defined here. They are used in ChainHeaders as
//! a way of providing cryptographically verifiable proof of a given agent
//! as having been the author of a given data entry.

/// Signature is meant in the classic cryptographic sense,
/// as a string which can be validated as having been signed
/// by the private key associated with a given public key
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Signature(String);

impl From<&'static str> for Signature {
    fn from(s: &str) -> Signature {
        Signature(s.to_owned())
    }
}

impl From<String> for Signature {
    fn from(s: String) -> Signature {
        Signature(s.to_owned())
    }
}