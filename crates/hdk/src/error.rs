//! This file contains defitions for Zome errors and also Zome Results.

use crate::{
    holochain_core_types::error::HolochainError, holochain_persistence_api::error::PersistenceError,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_json_derive::DefaultJson;
use holochain_wasmer_guest::*;
use serde_derive::{Deserialize, Serialize};
use std::{error::Error, fmt};
use holochain_core_types::validation::ValidationResult;

/// Error for DNA developers to use in their Zome code.
/// This does not have to be sent back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize, Deserialize, PartialEq, DefaultJson, Clone)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    Timeout,
}

impl From<ZomeApiError> for ValidationResult {
    fn from(e: ZomeApiError) -> Self {
        match e {
            // any abitrary zome string is a fail
            ZomeApiError::Internal(s) => Self::Fail(s),
            ZomeApiError::FunctionNotImplemented => Self::NotImplemented,
            ZomeApiError::Timeout => Self::Timeout,
        }
    }
}

impl From<ZomeApiError> for HolochainError {
    fn from(zome_api_error: ZomeApiError) -> Self {
        match zome_api_error {
            ZomeApiError::Timeout => HolochainError::Timeout,
            _ => HolochainError::Wasm(WasmError::Zome(zome_api_error.to_string())),
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

impl Error for ZomeApiError {}

impl fmt::Display for ZomeApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZomeApiError::Internal(msg) => write!(f, "{}", msg),
            ZomeApiError::FunctionNotImplemented => write!(f, "Function not implemented"),
            ZomeApiError::Timeout => write!(f, "Timeout"),
        }
    }
}

pub type ZomeApiResult<T> = Result<T, ZomeApiError>;

#[cfg(test)]
mod tests {

    use crate::error::{ZomeApiError, ZomeApiResult};
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
