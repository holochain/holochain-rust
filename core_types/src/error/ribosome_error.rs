use self::{RibosomeErrorCode::*, RibosomeReturnCode::*};
use crate::{error::HolochainError, json::JsonString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{convert::TryFrom, str::FromStr};

/// Enum of all possible RETURN codes that a Zome API Function could return.
/// Represents an encoded allocation of zero length with the return code as offset.
/// @see SinglePageAllocation
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
pub enum RibosomeReturnCode {
    Success,
    Failure(RibosomeErrorCode),
}

impl From<RibosomeReturnCode> for i32 {
    fn from(ribosome_return_code: RibosomeReturnCode) -> i32 {
        match ribosome_return_code {
            RibosomeReturnCode::Success => 0,
            RibosomeReturnCode::Failure(code) => code as i32,
        }
    }
}

impl From<RibosomeReturnCode> for u32 {
    fn from(ribosome_return_code: RibosomeReturnCode) -> u32 {
        match ribosome_return_code {
            RibosomeReturnCode::Success => 0,
            RibosomeReturnCode::Failure(code) => code as i32 as u32,
        }
    }
}

impl ToString for RibosomeReturnCode {
    fn to_string(&self) -> String {
        match self {
            Success => "Success".to_string(),
            Failure(code) => code.to_string(),
        }
    }
}

impl FromStr for RibosomeReturnCode {
    type Err = HolochainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.as_ref() {
            "Success" => RibosomeReturnCode::Success,
            _ => RibosomeReturnCode::Failure(s.parse()?),
        })
    }
}

impl From<RibosomeReturnCode> for JsonString {
    fn from(ribosome_return_code: RibosomeReturnCode) -> JsonString {
        JsonString::from(ribosome_return_code.to_string())
    }
}

impl TryFrom<JsonString> for RibosomeReturnCode {
    type Error = HolochainError;

    fn try_from(json_string: JsonString) -> Result<Self, Self::Error> {
        String::from(json_string).parse()
    }
}

impl RibosomeReturnCode {
    pub fn from_error(err_code: RibosomeErrorCode) -> Self {
        Failure(err_code)
    }

    pub fn from_offset(offset: u16) -> Self {
        match offset {
            0 => Success,
            _ => Failure(RibosomeErrorCode::from_offset(offset)),
        }
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
    pub fn from_offset(offset: u16) -> Self {
        match offset {
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

    pub fn from_return_code(ret_code: RibosomeReturnCode) -> Self {
        match ret_code {
            Success => unreachable!(),
            Failure(rib_err) => rib_err,
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
    fn ribosome_return_code_round_trip() {
        let oom =
            RibosomeReturnCode::from_offset(((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16);
        assert_eq!(Failure(RibosomeErrorCode::OutOfMemory), oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn ribosome_error_code_round_trip() {
        let oom =
            RibosomeErrorCode::from_offset(((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16);
        assert_eq!(RibosomeErrorCode::OutOfMemory, oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn error_conversion() {
        for code in 1..=10 {
            let mut err = RibosomeErrorCode::from_offset(code);

            let err_str = err.as_str().to_owned();

            err = err_str.parse().expect("unable to parse error");

            let inner_code = RibosomeReturnCode::from_error(err);

            let _one_int: i32 = inner_code.clone().into();
            let _another_int: u32 = inner_code.clone().into();
        }
    }

    #[test]
    #[should_panic]
    fn code_zero() {
        RibosomeErrorCode::from_offset(0);
    }
}
