//! This file contains defitions for Zome errors and also Zome Results.

use crate::holochain_core_types::{
    error::{HolochainError, RibosomeErrorCode},
    json::{JsonError, JsonString},
};
use holochain_wasm_utils::memory::allocation::AllocationError;
use std::{error::Error, fmt};

/// Error for DNA developers to use in their Zome code.
/// This does not have to be sent back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
    Timeout,
}

impl JsonError for ZomeApiError {}

impl From<ZomeApiError> for HolochainError {
    fn from(zome_api_error: ZomeApiError) -> Self {
        match zome_api_error {
            ZomeApiError::ValidationFailed(s) => HolochainError::ValidationFailed(s),
            _ => HolochainError::RibosomeFailed(zome_api_error.description().into()),
        }
    }
}

impl From<ZomeApiError> for String {
    fn from(zome_api_error: ZomeApiError) -> Self {
        zome_api_error.description().into()
    }
}

impl From<HolochainError> for ZomeApiError {
    fn from(holochain_error: HolochainError) -> Self {
        match holochain_error {
            HolochainError::ValidationFailed(s) => ZomeApiError::ValidationFailed(s),
            HolochainError::Timeout => ZomeApiError::Timeout,
            _ => ZomeApiError::Internal(holochain_error.description().into()),
        }
    }
}

impl From<!> for ZomeApiError {
    fn from(_: !) -> Self {
        unreachable!();
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

impl From<AllocationError> for ZomeApiError {
    fn from(allocation_error: AllocationError) -> ZomeApiError {
        match allocation_error {
            AllocationError::OutOfBounds => {
                ZomeApiError::Internal("Allocation out of bounds".into())
            }
            AllocationError::ZeroLength => ZomeApiError::Internal("Allocation zero length".into()),
            AllocationError::BadStackAlignment => {
                ZomeApiError::Internal("Allocation out of alignment with stack".into())
            }
            AllocationError::Serialization => {
                ZomeApiError::Internal("Allocation serialization failure".into())
            }
        }
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
            ZomeApiError::Timeout                 => "Timeout",
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

#[cfg(test)]
mod tests {

    use error::{ZomeApiError, ZomeApiResult};
    use holochain_core_types::json::JsonString;

    #[test]
    fn zome_api_result_json_result_round_trip_test() {
        let result: ZomeApiResult<String> = Err(ZomeApiError::FunctionNotImplemented);

        assert_eq!(
            JsonString::from(result),
            JsonString::from("{\"Err\":\"FunctionNotImplemented\"}"),
        );
    }
}
