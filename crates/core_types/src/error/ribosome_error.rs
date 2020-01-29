use self::{RibosomeReturnValue::*, RibosomeErrorCode::*};
use crate::error::HolochainError;
use holochain_json_api::{error::JsonError, json::JsonString};

use bits_n_pieces::u64_split_bits;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, str::FromStr};

/// size of the integer that encodes ribosome codes
pub type WasmAllocationInt = u64;
/// size of the integer that wasm sees
pub type RibosomeRuntimeBits = u64;
pub type RibosomeRuntimeArgBits = i64;
/// size of the integer that represents a ribosome code
pub type RibosomeCodeBits = u32;

// #[derive(Clone, Debug, PartialEq)]
// pub struct RibosomeEncodedAllocation(WasmAllocationInt);
//
// impl From<RibosomeEncodedAllocation> for WasmAllocationInt {
//     fn from(ribosome_memory_allocation: RibosomeEncodedAllocation) -> WasmAllocationInt {
//         ribosome_memory_allocation.0
//     }
// }
//
// impl From<WasmAllocationInt> for RibosomeEncodedAllocation {
//     fn from(i: WasmAllocationInt) -> Self {
//         Self(i)
//     }
// }
//
// impl ToString for RibosomeEncodedAllocation {
//     fn to_string(&self) -> String {
//         WasmAllocationInt::from(self.to_owned()).to_string()
//     }
// }

/// Represents all possible values passed to/from wasm functions
/// All wasm functions are I64 values
#[repr(u64)]
#[derive(Clone, Debug, PartialEq)]
pub enum RibosomeReturnValue {
    /// @TODO make this unambiguous or remove
    /// Contextually represents:
    /// - Function succeeded without any allocation
    /// - Empty/nil argument to a function
    /// - Zero length allocation (error)
    Success,
    /// A value that can be safely converted to a wasm allocation
    /// High bits represent offset, low bits represent length
    /// @see WasmAllocation
    Allocation(RibosomeEncodedAllocation),
    /// A value that should be interpreted as an error
    /// Low bits are zero, high bits map to an enum variant
    Failure(RibosomeErrorCode),
}

impl From<RibosomeReturnValue> for WasmAllocationInt {
    fn from(ribosome_return_code: RibosomeReturnValue) -> WasmAllocationInt {
        match ribosome_return_code {
            RibosomeReturnValue::Success => 0,
            RibosomeReturnValue::Allocation(allocation) => WasmAllocationInt::from(allocation),
            RibosomeReturnValue::Failure(code) => {
                code as RibosomeRuntimeBits as WasmAllocationInt
            }
        }
    }
}

// impl From<RibosomeReturnValue> for RibosomeRuntimeBits {
//     fn from(ribosome_return_code: RibosomeReturnValue) -> RibosomeRuntimeBits {
//         WasmAllocationInt::from(ribosome_return_code) as RibosomeRuntimeBits
//     }
// }

impl From<WasmAllocationInt> for RibosomeReturnValue {
    fn from(i: WasmAllocationInt) -> Self {
        if i == 0 {
            RibosomeReturnValue::Success
        } else {
            let (code_int, maybe_allocation_length) = u64_split_bits(i);
            if maybe_allocation_length == 0 {
                RibosomeReturnValue::Failure(RibosomeErrorCode::from_code_int(code_int))
            } else {
                RibosomeReturnValue::Allocation(RibosomeEncodedAllocation(i))
            }
        }
    }
}

impl ToString for RibosomeReturnValue {
    fn to_string(&self) -> String {
        match self {
            Success => "Success".to_string(),
            Allocation(allocation) => allocation.to_string(),
            Failure(code) => code.to_string(),
        }
    }
}

impl From<RibosomeReturnValue> for String {
    fn from(return_code: RibosomeReturnValue) -> String {
        return_code.to_string()
    }
}

impl FromStr for RibosomeReturnValue {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Success" => RibosomeReturnValue::Success,
            _ => RibosomeReturnValue::Failure(s.parse()?),
        })
    }
}

impl From<RibosomeReturnValue> for JsonString {
    fn from(ribosome_return_code: RibosomeReturnValue) -> JsonString {
        JsonString::from_json(&ribosome_return_code.to_string())
    }
}

impl From<HolochainError> for RibosomeReturnValue {
    fn from(error: HolochainError) -> Self {
        RibosomeReturnValue::Failure(RibosomeErrorCode::from(error))
    }
}

impl TryFrom<JsonString> for RibosomeReturnValue {
    type Error = HolochainError;

    fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
        String::from(json_string).parse()
    }
}

impl RibosomeReturnValue {
    pub fn from_error(err_code: RibosomeErrorCode) -> Self {
        Failure(err_code)
    }
}

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[repr(u64)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, DefaultJson, PartialOrd, Ord)]
#[rustfmt::skip]
pub enum RibosomeErrorCode {
    Unspecified                     = 1 << 32,
    ArgumentDeserializationFailed   = 2 << 32,
    OutOfMemory                     = 3 << 32,
    ReceivedWrongActionResult       = 4 << 32,
    CallbackFailed                  = 5 << 32,
    RecursiveCallForbidden          = 6 << 32,
    ResponseSerializationFailed     = 7 << 32,
    NotAnAllocation                 = 8 << 32,
    ZeroSizedAllocation             = 9 << 32,
    UnknownEntryType                = 10 << 32,
    MismatchWasmCallDataType        = 11 << 32,
    EntryNotFound                   = 12 << 32,
    WorkflowFailed                  = 13 << 32,
}

#[rustfmt::skip]
impl RibosomeErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Unspecified                     => "Unspecified",
            ArgumentDeserializationFailed   => "Argument deserialization failed",
            OutOfMemory                     => "Out of memory",
            ReceivedWrongActionResult       => "Received wrong action result",
            CallbackFailed                  => "Callback failed",
            RecursiveCallForbidden          => "Recursive call forbidden",
            ResponseSerializationFailed     => "Response serialization failed",
            NotAnAllocation                 => "Not an allocation",
            ZeroSizedAllocation             => "Zero-sized allocation",
            UnknownEntryType                => "Unknown entry type",
            MismatchWasmCallDataType        => "Mismatched WasmCallData type",
            EntryNotFound                   => "Entry Could Not Be Found",
            WorkflowFailed                  => "Workflow failed",
        }
    }
}

impl From<HolochainError> for RibosomeErrorCode {
    fn from(error: HolochainError) -> RibosomeErrorCode {
        // the mapping between HolochainError and RibosomeErrorCode is pretty poor overall
        match error {
            HolochainError::ErrorGeneric(_) => RibosomeErrorCode::Unspecified,
            HolochainError::CryptoError(_) => RibosomeErrorCode::Unspecified,
            HolochainError::NotImplemented(_) => RibosomeErrorCode::CallbackFailed,
            HolochainError::LoggingError => RibosomeErrorCode::Unspecified,
            HolochainError::DnaMissing => RibosomeErrorCode::Unspecified,
            HolochainError::Dna(_) => RibosomeErrorCode::Unspecified,
            HolochainError::IoError(_) => RibosomeErrorCode::Unspecified,
            HolochainError::SerializationError(_) => {
                RibosomeErrorCode::ArgumentDeserializationFailed
            }
            HolochainError::InvalidOperationOnSysEntry => RibosomeErrorCode::UnknownEntryType,
            HolochainError::CapabilityCheckFailed => RibosomeErrorCode::Unspecified,
            HolochainError::ValidationFailed(_) => RibosomeErrorCode::CallbackFailed,
            HolochainError::ValidationPending => RibosomeErrorCode::Unspecified,
            HolochainError::Ribosome(e) => e,
            HolochainError::RibosomeFailed(_) => RibosomeErrorCode::CallbackFailed,
            HolochainError::ConfigError(_) => RibosomeErrorCode::Unspecified,
            HolochainError::Timeout => RibosomeErrorCode::Unspecified,
            HolochainError::InitializationFailed(_) => RibosomeErrorCode::Unspecified,
            HolochainError::LifecycleError(_) => RibosomeErrorCode::Unspecified,
            HolochainError::DnaHashMismatch(_, _) => RibosomeErrorCode::Unspecified,
            HolochainError::EntryNotFoundLocally => RibosomeErrorCode::Unspecified,
            HolochainError::EntryIsPrivate => RibosomeErrorCode::Unspecified,
            HolochainError::List(_) => RibosomeErrorCode::Unspecified,
        }
    }
}

impl ToString for RibosomeErrorCode {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl From<RibosomeErrorCode> for String {
    fn from(ribosome_error_code: RibosomeErrorCode) -> Self {
        ribosome_error_code.to_string()
    }
}

impl RibosomeErrorCode {
    pub fn from_code_int(code: RibosomeCodeBits) -> Self {
        match code {
            0 => panic!(format!("RibosomeErrorCode == {:?} encountered", code)),
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

    pub fn from_return_code(ret_code: RibosomeReturnValue) -> Self {
        match ret_code {
            Failure(rib_err) => rib_err,
            _ => panic!(format!(
                "RibosomeReturnValue == {:?} encountered",
                ret_code
            )),
        }
    }
}

// @TODO review this serialization, can it be an i32 instead of a full string?
// @see https://github.com/holochain/holochain-rust/issues/591
impl Serialize for RibosomeErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RibosomeErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(RibosomeErrorCode::from_str(&s).expect("could not deserialize RibosomeErrorCode"))
    }
}

impl FromStr for RibosomeErrorCode {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(RibosomeErrorCode::Unspecified),
            "Argument deserialization failed" => {
                Ok(RibosomeErrorCode::ArgumentDeserializationFailed)
            }
            "Out of memory" => Ok(RibosomeErrorCode::OutOfMemory),
            "Received wrong action result" => Ok(RibosomeErrorCode::ReceivedWrongActionResult),
            "Callback failed" => Ok(RibosomeErrorCode::CallbackFailed),
            "Recursive call forbidden" => Ok(RibosomeErrorCode::RecursiveCallForbidden),
            "Response serialization failed" => Ok(RibosomeErrorCode::ResponseSerializationFailed),
            "Not an allocation" => Ok(RibosomeErrorCode::NotAnAllocation),
            "Zero-sized allocation" => Ok(RibosomeErrorCode::ZeroSizedAllocation),
            "Unknown entry type" => Ok(RibosomeErrorCode::UnknownEntryType),
            "Entry Could Not Be Found" => Ok(EntryNotFound),
            "Workflow failed" => Ok(WorkflowFailed),
            _ => Err(HolochainError::ErrorGeneric(String::from(
                "Unknown RibosomeErrorCode",
            ))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ribosome_error_code_round_trip() {
        let oom = RibosomeErrorCode::from_code_int(
            ((RibosomeErrorCode::OutOfMemory as u64) >> 32) as RibosomeCodeBits,
        );
        assert_eq!(RibosomeErrorCode::OutOfMemory, oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn error_conversion() {
        // TODO could use strum crate to iteratively
        // gather all known codes.
        for code in 1..=13 {
            let mut err = RibosomeErrorCode::from_code_int(code);

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
        RibosomeErrorCode::from_code_int(0);
    }
}
