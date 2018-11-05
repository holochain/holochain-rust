use std::{error::Error, fmt};
use holochain_core_types::error::HolochainError;
use holochain_core_types::json::JsonString;
use holochain_core_types::error::RibosomeErrorCode;

/// Error for DNA developers to use in their zome code.
/// They do not have to send this error back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl From<ZomeApiError> for HolochainError {
    fn from(zome_api_error: ZomeApiError) -> Self {
        match zome_api_error {
            ZomeApiError::ValidationFailed(s) => HolochainError::ValidationFailed(s),
            _ => HolochainError::RibosomeFailed(zome_api_error.description().into()),
        }
    }
}

impl From<HolochainError> for ZomeApiError {
    fn from(holochain_error: HolochainError) -> Self {
        match holochain_error {
            HolochainError::ValidationFailed(s) => ZomeApiError::ValidationFailed(s),
            _ => ZomeApiError::Internal(holochain_error.description().into()),
        }
    }
}

impl From<!> for ZomeApiError {
    fn from(_: !) -> Self {
        unreachable!();
    }
}

impl From<ZomeApiError> for JsonString {
    fn from(zome_api_error: ZomeApiError) -> JsonString {
        JsonString::from(json!({ "error": zome_api_error }))
    }
}

impl From<String> for ZomeApiError {
    fn from(s: String) -> ZomeApiError {
        ZomeApiError::Internal(s)
    }
}

impl From<RibosomeErrorCode> for ZomeApiError {
    fn from(ribosome_error_code: RibosomeErrorCode) -> ZomeApiError {
        ZomeApiError::from(ribosome_error_code.to_string())
    }
}

impl Error for ZomeApiError {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn description(&self) -> &str {
        match self {
            ZomeApiError::Internal(msg)           => &msg,
            ZomeApiError::FunctionNotImplemented  => "Function not implemented",
            ZomeApiError::HashNotFound            => "Hash not found",
            ZomeApiError::ValidationFailed(msg)   => &msg,
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

pub type ZomeApiResult<T> = Result<T, ZomeApiError>;
