use error::HolochainError;

pub trait ToJson {
    /// serialize self to a canonical JSON string
    fn to_json(&self) -> Result<String, HolochainError>;
}

pub trait FromJson
    where Self: Sized {
    /// deserialize a Pair from a canonical JSON string
    fn from_json(s: &str) -> Result<Self, HolochainError>;
}

pub trait RoundTripJson: ToJson + FromJson {}
