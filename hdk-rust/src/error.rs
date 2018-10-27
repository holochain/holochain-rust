use self::ZomeApiError::*;
use serde_json;
use std::{error::Error, fmt};

pub type ZomeApiResult<T> = Result<T, ZomeApiError>;

/// Error for DNA developers to use in their zome code.
/// They do not have to send this error back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl ZomeApiError {
    pub fn to_json(&self) -> serde_json::Value {
        json!({ "error": self })
    }
}

impl Error for ZomeApiError {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn description(&self) -> &str {
        match self {
            Internal(msg)           => &msg,
            FunctionNotImplemented  => "Function not implemented",
            HashNotFound            => "Hash not found",
            ValidationFailed(msg)   => &msg,
        }
    }
}

impl fmt::Display for ZomeApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO seems weird to use debug for display
        // replacing {:?} with {} gives a stack overflow on to_string() (there's a test for this)
        // what is the right way to do this?
        // @see https://github.com/holochain/holochain-rust/issues/223
        write!(f, "{:?}", self)
    }
}
