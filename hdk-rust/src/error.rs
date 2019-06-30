//! This file contains defitions for Zome errors and also Zome Results.

use crate::holochain_core_types::error::{HolochainError, RibosomeErrorCode};

use crate::holochain_persistence_api::error::PersistenceError;
use holochain_json_api::{error::JsonError, json::JsonString};

use holochain_wasm_utils::memory::allocation::AllocationError;
use std::{error::Error, fmt};

/// Error for DNA developers to use in their Zome code.
/// This does not have to be sent back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
    Timeout,
}

impl From<ZomeApiError> for HolochainError {
    fn from(zome_api_error: ZomeApiError) -> Self {
        match zome_api_error {
            ZomeApiError::ValidationFailed(s) => HolochainError::ValidationFailed(s),
            _ => HolochainError::RibosomeFailed(zome_api_error.to_string()),
        }
    }
}

impl From<ZomeApiError> for String {
    fn from(zome_api_error: ZomeApiError) -> Self {
        zome_api_error.to_string()
    }
}

impl From<HolochainError> for ZomeApiError {
    fn from(holochain_error: HolochainError) -> Self {
        match holochain_error {
            HolochainError::ValidationFailed(s) => ZomeApiError::ValidationFailed(s),
            HolochainError::Timeout => ZomeApiError::Timeout,
            _ => ZomeApiError::Internal(holochain_error.to_string()),
        }
    }
}

impl From<PersistenceError> for ZomeApiError {
    fn from(persistence_error: PersistenceError) -> Self {
        let holochain_error: HolochainError = persistence_error.into();
        holochain_error.into()
    }
}

impl From<JsonError> for ZomeApiError {
    fn from(json_error: JsonError) -> Self {
        let holochain_error: HolochainError = json_error.into();
        holochain_error.into()
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

impl Error for ZomeApiError {}

impl fmt::Display for ZomeApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZomeApiError::Internal(msg) => write!(f, "{}", msg),
            ZomeApiError::FunctionNotImplemented => write!(f, "Function not implemented"),
            ZomeApiError::HashNotFound => write!(f, "Hash not found"),
            ZomeApiError::ValidationFailed(msg) => write!(f, "{}", msg),
            ZomeApiError::Timeout => write!(f, "Timeout"),
        }
    }
}

pub type ZomeApiResult<T> = Result<T, ZomeApiError>;

#[cfg(test)]
mod tests {

    use error::{ZomeApiError, ZomeApiResult};
    use holochain_json_api::json::JsonString;

    #[test]
    fn zome_api_result_json_result_round_trip_test() {
        let result: ZomeApiResult<String> = Err(ZomeApiError::FunctionNotImplemented);

        assert_eq!(
            JsonString::from(result),
            JsonString::from_json("{\"Err\":\"FunctionNotImplemented\"}"),
        );
    }
}
