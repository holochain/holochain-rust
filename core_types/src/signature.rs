#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Signature(String);

// Signatures to match Sources, zip together
pub type Signatures = Vec<Signature>;

impl From<&'static str> for Signature {
    fn from(s: &str) -> Signature {
        Signature(s.to_owned())
    }
}

pub fn test_signatures() -> Signatures {
    vec![Signature::from("fake-signature")]
}

pub fn test_signature_b() -> Signature {
    Signature::from("another-fake-signature")
}

pub fn test_signature_c() -> Signature {
    Signature::from("sig-c")
}
