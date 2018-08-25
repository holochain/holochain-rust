use serde_json;
use error::HolochainError;

pub trait ToJson {
    /// serialize self to a canonical JSON string
    fn to_json(&self) -> Result<String, HolochainError> {
        // @TODO error handling
        // @see https://github.com/holochain/holochain-rust/issues/168
        let result = serde_json::to_string(&self);
        match result {
            Ok(r) => Ok(r),
            Err(e) => HolochainError::SerializationError(e),
        }
    }
}

pub trait FromJson
    where Self: Sized {
    /// deserialize a Pair from a canonical JSON string
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        let result: Self = serde_json::from_str(s);
        match result {
            Ok(r) => Ok(r),
            Err(e) => HolochainError::SerializationError(e),
        }
    }
}

pub trait RoundTripJson: ToJson + FromJson {}
