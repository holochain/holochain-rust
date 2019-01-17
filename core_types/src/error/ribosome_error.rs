use self::{RibosomeEncodedValue::*, RibosomeErrorCode::*};
use crate::{error::HolochainError, json::JsonString};
use bits_n_pieces::u32_split_bits;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, str::FromStr};

/// size of the integer that encodes ribosome codes
pub type RibosomeEncodingBits = u32;
/// size of the integer that wasm sees
pub type RibosomeRuntimeBits = i32;
/// size of the integer that represents a ribosome code
pub type RibosomeCodeBits = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct RibosomeEncodedAllocation(RibosomeEncodingBits);

impl From<RibosomeEncodedAllocation> for RibosomeEncodingBits {
    fn from(ribosome_memory_allocation: RibosomeEncodedAllocation) -> RibosomeEncodingBits {
        ribosome_memory_allocation.0
    }
}

impl From<RibosomeEncodingBits> for RibosomeEncodedAllocation {
    fn from(i: RibosomeEncodingBits) -> Self {
        Self(i)
    }
}

impl ToString for RibosomeEncodedAllocation {
    fn to_string(&self) -> String {
        RibosomeEncodingBits::from(self.to_owned()).to_string()
    }
}

/// Enum of all possible RETURN codes that a Zome API Function could return.
/// Represents an encoded allocation of zero length with the return code as offset.
/// @see SinglePageAllocation
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
pub enum RibosomeEncodedValue {
    Success,
    Allocation(RibosomeEncodedAllocation),
    Failure(RibosomeErrorCode),
}

impl From<RibosomeEncodedValue> for RibosomeEncodingBits {
    fn from(ribosome_return_code: RibosomeEncodedValue) -> RibosomeEncodingBits {
        match ribosome_return_code {
            RibosomeEncodedValue::Success => 0,
            RibosomeEncodedValue::Allocation(allocation) => RibosomeEncodingBits::from(allocation),
            RibosomeEncodedValue::Failure(code) => {
                code as RibosomeRuntimeBits as RibosomeEncodingBits
            }
        }
    }
}

impl From<RibosomeEncodedValue> for RibosomeRuntimeBits {
    fn from(ribosome_return_code: RibosomeEncodedValue) -> RibosomeRuntimeBits {
        RibosomeEncodingBits::from(ribosome_return_code) as RibosomeRuntimeBits
    }
}

impl From<RibosomeEncodingBits> for RibosomeEncodedValue {
    fn from(i: RibosomeEncodingBits) -> Self {
        if i == 0 {
            RibosomeEncodedValue::Success
        } else {
            let (code_int, maybe_allocation_length) = u32_split_bits(i);
            if maybe_allocation_length == 0 {
                RibosomeEncodedValue::Failure(RibosomeErrorCode::from_code_int(code_int))
            } else {
                RibosomeEncodedValue::Allocation(RibosomeEncodedAllocation(i))
            }
        }
    }
}

impl ToString for RibosomeEncodedValue {
    fn to_string(&self) -> String {
        match self {
            Success => "Success".to_string(),
            Allocation(allocation) => allocation.to_string(),
            Failure(code) => code.to_string(),
        }
    }
}

impl From<RibosomeEncodedValue> for String {
    fn from(return_code: RibosomeEncodedValue) -> String {
        return_code.to_string()
    }
}

impl FromStr for RibosomeEncodedValue {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_ref() {
            "Success" => RibosomeEncodedValue::Success,
            _ => RibosomeEncodedValue::Failure(s.parse()?),
        })
    }
}

impl From<RibosomeEncodedValue> for JsonString {
    fn from(ribosome_return_code: RibosomeEncodedValue) -> JsonString {
        JsonString::from(ribosome_return_code.to_string())
    }
}

impl From<HolochainError> for RibosomeEncodedValue {
    fn from(error: HolochainError) -> Self {
        RibosomeEncodedValue::Failure(RibosomeErrorCode::from(error))
    }
}

impl TryFrom<JsonString> for RibosomeEncodedValue {
    type Error = HolochainError;

    fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
        String::from(json_string).parse()
    }
}

impl RibosomeEncodedValue {
    pub fn from_error(err_code: RibosomeErrorCode) -> Self {
        Failure(err_code)
    }
}

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[repr(u32)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, DefaultJson)]
#[rustfmt::skip]
pub enum RibosomeErrorCode {
    Unspecified                     = 1 << 16,
    ArgumentDeserializationFailed   = 2 << 16,
    OutOfMemory                     = 3 << 16,
    ReceivedWrongActionResult       = 4 << 16,
    CallbackFailed                  = 5 << 16,
    RecursiveCallForbidden          = 6 << 16,
    ResponseSerializationFailed     = 7 << 16,
    NotAnAllocation                 = 8 << 16,
    ZeroSizedAllocation             = 9 << 16,
    UnknownEntryType                = 10 << 16,
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
        }
    }
}

impl From<HolochainError> for RibosomeErrorCode {
    fn from(error: HolochainError) -> RibosomeErrorCode {
        // the mapping between HolochainError and RibosomeErrorCode is pretty poor overall
        match error {
            HolochainError::ErrorGeneric(_) => RibosomeErrorCode::Unspecified,
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
            HolochainError::Ribosome(e) => e,
            HolochainError::RibosomeFailed(_) => RibosomeErrorCode::CallbackFailed,
            HolochainError::ConfigError(_) => RibosomeErrorCode::Unspecified,
            HolochainError::Timeout => RibosomeErrorCode::Unspecified,
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
            0 => unreachable!(),
            2 => ArgumentDeserializationFailed,
            3 => OutOfMemory,
            4 => ReceivedWrongActionResult,
            5 => CallbackFailed,
            6 => RecursiveCallForbidden,
            7 => ResponseSerializationFailed,
            8 => NotAnAllocation,
            9 => ZeroSizedAllocation,
            10 => UnknownEntryType,
            1 | _ => Unspecified,
        }
    }

    pub fn from_return_code(ret_code: RibosomeEncodedValue) -> Self {
        match ret_code {
            Failure(rib_err) => rib_err,
            _ => unreachable!(),
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
            ((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16,
        );
        assert_eq!(RibosomeErrorCode::OutOfMemory, oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn error_conversion() {
        for code in 1..=10 {
            let mut err = RibosomeErrorCode::from_code_int(code);

            let err_str = err.as_str().to_owned();

            err = err_str.parse().expect("unable to parse error");

            let inner_code = RibosomeEncodedValue::from_error(err);

            let _one_int: i32 = inner_code.clone().into();
            let _another_int: u32 = inner_code.clone().into();
        }
    }

    #[test]
    #[should_panic]
    fn code_zero() {
        RibosomeErrorCode::from_code_int(0);
    }
}
