use self::{RibosomeError::*};
use crate::error::HolochainError;
use holochain_json_api::{error::JsonError, json::JsonString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{str::FromStr};

/// size of the integer that represents a ribosome code
pub type RibosomeCodeBits = u32;

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[derive(Clone, Debug, PartialEq, Eq, Hash, DefaultJson, PartialOrd, Ord)]
#[rustfmt::skip]
pub enum RibosomeError {
    Unspecified,
    ArgumentDeserializationFailed,
    OutOfMemory,
    ReceivedWrongActionResult,
    CallbackFailed,
    RecursiveCallForbidden,
    ResponseSerializationFailed,
    NotAnAllocation,
    ZeroSizedAllocation,
    UnknownEntryType,
    MismatchWasmCallDataType,
    EntryNotFound,
    WorkflowFailed,
    // something to do with zome logic
    Zome(String),
}

#[derive(Debug)]
pub enum RibosomeResult {
    Value(JsonString),
    Error(RibosomeError),
}

impl From<HolochainError> for RibosomeError {
    fn from(error: HolochainError) -> RibosomeError {
        // the mapping between HolochainError and RibosomeError is pretty poor overall
        match error {
            HolochainError::ErrorGeneric(_) => RibosomeError::Unspecified,
            HolochainError::CryptoError(_) => RibosomeError::Unspecified,
            HolochainError::NotImplemented(_) => RibosomeError::CallbackFailed,
            HolochainError::LoggingError => RibosomeError::Unspecified,
            HolochainError::DnaMissing => RibosomeError::Unspecified,
            HolochainError::Dna(_) => RibosomeError::Unspecified,
            HolochainError::IoError(_) => RibosomeError::Unspecified,
            HolochainError::SerializationError(_) => {
                RibosomeError::ArgumentDeserializationFailed
            }
            HolochainError::InvalidOperationOnSysEntry => RibosomeError::UnknownEntryType,
            HolochainError::CapabilityCheckFailed => RibosomeError::Unspecified,
            HolochainError::ValidationFailed(_) => RibosomeError::CallbackFailed,
            HolochainError::ValidationPending => RibosomeError::Unspecified,
            HolochainError::Ribosome(e) => e,
            HolochainError::RibosomeFailed(_) => RibosomeError::CallbackFailed,
            HolochainError::ConfigError(_) => RibosomeError::Unspecified,
            HolochainError::Timeout => RibosomeError::Unspecified,
            HolochainError::InitializationFailed(_) => RibosomeError::Unspecified,
            HolochainError::LifecycleError(_) => RibosomeError::Unspecified,
            HolochainError::DnaHashMismatch(_, _) => RibosomeError::Unspecified,
            HolochainError::EntryNotFoundLocally => RibosomeError::Unspecified,
            HolochainError::EntryIsPrivate => RibosomeError::Unspecified,
            HolochainError::List(_) => RibosomeError::Unspecified,
        }
    }
}

impl From<RibosomeError> for String {
    fn from(ribosome_error_code: RibosomeError) -> Self {
        ribosome_error_code.to_string()
    }
}

impl RibosomeError {
    pub fn from_code_int(code: RibosomeCodeBits) -> Self {
        match code {
            0 => panic!(format!("RibosomeError == {:?} encountered", code)),
            2 => ArgumentDeserializationFailed,
            3 => OutOfMemory,
            4 => ReceivedWrongActionResult,
            5 => CallbackFailed,
            6 => RecursiveCallForbidden,
            7 => ResponseSerializationFailed,
            8 => NotAnAllocation,
            9 => ZeroSizedAllocation,
            10 => UnknownEntryType,
            12 => EntryNotFound,
            13 => WorkflowFailed,
            1 | _ => Unspecified,
        }
    }
}

// @TODO review this serialization, can it be an i32 instead of a full string?
// @see https://github.com/holochain/holochain-rust/issues/591
impl Serialize for RibosomeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RibosomeError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(RibosomeError::from_str(&s).expect("could not deserialize RibosomeError"))
    }
}

impl FromStr for RibosomeError {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(RibosomeError::Unspecified),
            "Argument deserialization failed" => {
                Ok(RibosomeError::ArgumentDeserializationFailed)
            }
            "Out of memory" => Ok(RibosomeError::OutOfMemory),
            "Received wrong action result" => Ok(RibosomeError::ReceivedWrongActionResult),
            "Callback failed" => Ok(RibosomeError::CallbackFailed),
            "Recursive call forbidden" => Ok(RibosomeError::RecursiveCallForbidden),
            "Response serialization failed" => Ok(RibosomeError::ResponseSerializationFailed),
            "Not an allocation" => Ok(RibosomeError::NotAnAllocation),
            "Zero-sized allocation" => Ok(RibosomeError::ZeroSizedAllocation),
            "Unknown entry type" => Ok(RibosomeError::UnknownEntryType),
            "Entry Could Not Be Found" => Ok(EntryNotFound),
            "Workflow failed" => Ok(WorkflowFailed),
            _ => Err(HolochainError::ErrorGeneric(String::from(
                "Unknown RibosomeError",
            ))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ribosome_error_code_round_trip() {
        let oom = RibosomeError::from_code_int(
            ((RibosomeError::OutOfMemory as u64) >> 32) as RibosomeCodeBits,
        );
        assert_eq!(RibosomeError::OutOfMemory, oom);
        assert_eq!(RibosomeError::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn error_conversion() {
        // TODO could use strum crate to iteratively
        // gather all known codes.
        for code in 1..=13 {
            let mut err = RibosomeError::from_code_int(code);

            let err_str = err.as_str().to_owned();

            err = err_str.parse().expect("unable to parse error");

            let inner_code = RibosomeReturnValue::from_error(err);

            let _one_int: u64 = inner_code.clone().into();
            let _another_int: u64 = inner_code.clone().into();
        }
    }

    #[test]
    #[should_panic]
    fn code_zero() {
        RibosomeError::from_code_int(0);
    }
}
