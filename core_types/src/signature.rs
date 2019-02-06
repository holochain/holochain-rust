//! The Signature type is defined here. They are used in ChainHeaders as
//! a way of providing cryptographically verifiable proof of a given agent
//! as having been the author of a given data entry.

use crate::cas::content::Address;

/// Provenance is a tuple of initiating agent and signature of some item being signed
/// this type is used in headers and in capability requests where the item being signed
/// is implicitly known by context
pub type Provenance = (Address, Signature);
/*impl Provenance {
    fn source(&self) -> Address {
        self.1.clone()
    }
}*/
/// Signature is meant in the classic cryptographic sense,
/// as a string which can be validated as having been signed
/// by the private key associated with a given public key
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct Signature(String);

impl Signature {
    pub fn fake() -> Signature {
        test_signature()
    }
}

impl From<&'static str> for Signature {
    fn from(s: &str) -> Signature {
        Signature(s.to_owned())
    }
}

impl From<String> for Signature {
    fn from(s: String) -> Signature {
        Signature(s)
    }
}

pub fn test_signatures() -> Vec<Signature> {
    vec![test_signature()]
}

pub fn test_signature() -> Signature {
    Signature::from("fake-signature")
}

pub fn test_signature_b() -> Signature {
    Signature::from("another-fake-signature")
}

pub fn test_signature_c() -> Signature {
    Signature::from("sig-c")
}
