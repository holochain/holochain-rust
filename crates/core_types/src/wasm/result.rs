use self::WasmError::*;
use crate::error::HolochainError;
use holochain_json_api::{error::JsonError, json::JsonString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// size of the integer that represents a ribosome code
pub type RibosomeCodeBits = u32;

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[derive(Clone, Debug, PartialEq, Eq, Hash, DefaultJson, PartialOrd, Ord)]
#[rustfmt::skip]
pub enum WasmError {
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
    // something to do with zome logic that we don't know about
    Zome(String),
}

#[derive(Debug, Serialize, Deserialize, DefaultJson)]
pub enum WasmResult {
    Ok(JsonString),
    Err(WasmError),
}

impl ToString for WasmError {
    fn to_string(&self) -> String {
        match self {
            WasmError::Unspecified => "Unspecified",
            WasmError::ArgumentDeserializationFailed => "ArgumentDeserializationFailed",
            WasmError::OutOfMemory => "OutOfMemory",
            WasmError::ReceivedWrongActionResult => "ReceivedWrongActionResult",
            WasmError::CallbackFailed => "CallbackFailed",
            WasmError::RecursiveCallForbidden => "RecursiveCallForbidden",
            WasmError::ResponseSerializationFailed => "ResponseSerializationFailed",
            WasmError::NotAnAllocation => "NotAnAllocation",
            WasmError::ZeroSizedAllocation => "ZeroSizedAllocation",
            WasmError::UnknownEntryType => "UnknownEntryType",
            WasmError::MismatchWasmCallDataType => "MismatchWasmCallDataType",
            WasmError::EntryNotFound => "EntryNotFound",
            WasmError::WorkflowFailed => "WorkflowFailed",
            WasmError::Zome(s) => s,
        }
        .into()
    }
}

impl From<HolochainError> for WasmError {
    fn from(error: HolochainError) -> WasmError {
        // the mapping between HolochainError and WasmError is pretty poor overall
        match error {
            HolochainError::ErrorGeneric(_) => WasmError::Unspecified,
            HolochainError::CryptoError(_) => WasmError::Unspecified,
            HolochainError::NotImplemented(_) => WasmError::CallbackFailed,
            HolochainError::LoggingError => WasmError::Unspecified,
            HolochainError::DnaMissing => WasmError::Unspecified,
            HolochainError::Dna(_) => WasmError::Unspecified,
            HolochainError::IoError(_) => WasmError::Unspecified,
            HolochainError::SerializationError(_) => WasmError::ArgumentDeserializationFailed,
            HolochainError::InvalidOperationOnSysEntry => WasmError::UnknownEntryType,
            HolochainError::CapabilityCheckFailed => WasmError::Unspecified,
            HolochainError::ValidationFailed(_) => WasmError::CallbackFailed,
            HolochainError::ValidationPending => WasmError::Unspecified,
            HolochainError::Wasm(e) => e,
            HolochainError::ConfigError(_) => WasmError::Unspecified,
            HolochainError::Timeout => WasmError::Unspecified,
            HolochainError::InitializationFailed(_) => WasmError::Unspecified,
            HolochainError::LifecycleError(_) => WasmError::Unspecified,
            HolochainError::DnaHashMismatch(_, _) => WasmError::Unspecified,
            HolochainError::EntryNotFoundLocally => WasmError::Unspecified,
            HolochainError::EntryIsPrivate => WasmError::Unspecified,
            HolochainError::List(_) => WasmError::Unspecified,
        }
    }
}

impl From<WasmError> for String {
    fn from(ribosome_error_code: WasmError) -> Self {
        ribosome_error_code.to_string()
    }
}

// @TODO review this serialization, can it be an i32 instead of a full string?
// @see https://github.com/holochain/holochain-rust/issues/591
impl Serialize for WasmError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WasmError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(WasmError::from_str(&s).expect("could not deserialize WasmError"))
    }
}

impl FromStr for WasmError {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(WasmError::Unspecified),
            "Argument deserialization failed" => Ok(WasmError::ArgumentDeserializationFailed),
            "Out of memory" => Ok(WasmError::OutOfMemory),
            "Received wrong action result" => Ok(WasmError::ReceivedWrongActionResult),
            "Callback failed" => Ok(WasmError::CallbackFailed),
            "Recursive call forbidden" => Ok(WasmError::RecursiveCallForbidden),
            "Response serialization failed" => Ok(WasmError::ResponseSerializationFailed),
            "Not an allocation" => Ok(WasmError::NotAnAllocation),
            "Zero-sized allocation" => Ok(WasmError::ZeroSizedAllocation),
            "Unknown entry type" => Ok(WasmError::UnknownEntryType),
            "Entry Could Not Be Found" => Ok(EntryNotFound),
            "Workflow failed" => Ok(WorkflowFailed),
            _ => Err(HolochainError::ErrorGeneric(String::from(
                "Unknown WasmError",
            ))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ribosome_error_code_round_trip() {
        let oom =
            WasmError::from_code_int(((WasmError::OutOfMemory as u64) >> 32) as RibosomeCodeBits);
        assert_eq!(WasmError::OutOfMemory, oom);
        assert_eq!(WasmError::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn error_conversion() {
        // TODO could use strum crate to iteratively
        // gather all known codes.
        for code in 1..=13 {
            let mut err = WasmError::from_code_int(code);

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
        WasmError::from_code_int(0);
    }
}
