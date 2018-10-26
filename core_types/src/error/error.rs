use json::*;
use self::HolochainError::*;
use error::DnaError;
use futures::channel::oneshot::Canceled as FutureCanceled;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Error as SerdeError};
use std::{
    convert::TryFrom,
    error::Error,
    fmt,
    io::{self, Error as IoError},
    str::FromStr,
};

/// Enum holding all Holochain specific errors
#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum HolochainError {
    ErrorGeneric(String),
    NotImplemented,
    LoggingError,
    DnaMissing,
    Dna(DnaError),
    IoError(String),
    SerializationError(String),
    InvalidOperationOnSysEntry,
    DoesNotHaveCapabilityToken,
    ValidationFailed(String),
    RibosomeFailed(String),
}

pub type HcResult<T> = Result<T, HolochainError>;

impl HolochainError {
    pub fn new(msg: &str) -> HolochainError {
        HolochainError::ErrorGeneric(msg.to_string())
    }
}

impl From<HolochainError> for JsonString {
    fn from(error: HolochainError) -> JsonString {
        JsonString::from(format!("{{\"error\":\"{}\"}}", error.description()))
    }
}

impl fmt::Display for HolochainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO seems weird to use debug for display
        // replacing {:?} with {} gives a stack overflow on to_string() (there's a test for this)
        // what is the right way to do this?
        // @see https://github.com/holochain/holochain-rust/issues/223
        write!(f, "{:?}", self)
    }
}

impl Error for HolochainError {
    fn description(&self) -> &str {
        match self {
            ErrorGeneric(err_msg) => &err_msg,
            NotImplemented => "not implemented",
            LoggingError => "logging failed",
            DnaMissing => "DNA is missing",
            Dna(dna_err) => dna_err.description(),
            IoError(err_msg) => &err_msg,
            SerializationError(err_msg) => &err_msg,
            InvalidOperationOnSysEntry => "operation cannot be done on a system entry type",
            DoesNotHaveCapabilityToken => "Caller does not have Capability to make that call",
            ValidationFailed(fail_msg) => &fail_msg,
            RibosomeFailed(fail_msg) => &fail_msg,
        }
    }
}

/// standard strings for std io errors
fn reason_for_io_error(error: &IoError) -> String {
    match error.kind() {
        io::ErrorKind::InvalidData => format!("contains invalid data: {}", error),
        io::ErrorKind::PermissionDenied => format!("missing permissions to read: {}", error),
        _ => format!("unexpected error: {}", error),
    }
}

impl From<IoError> for HolochainError {
    fn from(error: IoError) -> Self {
        HolochainError::IoError(reason_for_io_error(&error))
    }
}

impl From<SerdeError> for HolochainError {
    fn from(error: SerdeError) -> Self {
        HolochainError::SerializationError(error.to_string())
    }
}

impl From<FutureCanceled> for HolochainError {
    fn from(_: FutureCanceled) -> Self {
        HolochainError::ErrorGeneric("Failed future".to_string())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RibosomeErrorReport {
    pub description: String,
    pub file_name: String,
    pub line: String,
    // TODO #395 - Add advance error debugging info
    // pub stack_trace: Backtrace
}

impl fmt::Display for RibosomeErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ribosome error: {}\n  --> {}:{}\n",
            self.description, self.file_name, self.line,
        )
    }
}

impl From<RibosomeErrorReport> for String {
    fn from(ribosome_error_report: RibosomeErrorReport) -> String {
        ribosome_error_report.to_string()
    }
}

impl TryFrom<RibosomeErrorReport> for JsonString {
    type Error = HolochainError;
    fn try_from(v: RibosomeErrorReport) -> JsonResult {
        default_try_to_json(v)
    }
}

// impl TryFrom<JsonString> for RibosomeErrorReport {
//     type Error = HolochainError;
//     fn try_from(j: JsonString) -> JsonResult {
//         default_try_from_json(j)
//     }
// }

// impl From<RibosomeErrorReport> for JsonString {
//     fn from(ribosome_error_report: RibosomeErrorReport) -> JsonString {
//         JsonString::from(RawString::from(String::from(ribosome_error_report)))
//     }
// }

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

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
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

impl ToString for RibosomeReturnCode {
    fn to_string(&self) -> String {
        match self {
            RibosomeReturnCode::Success => "Success".to_string(),
            RibosomeReturnCode::Failure(code) => code.to_string(),
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

#[cfg_attr(rustfmt, rustfmt_skip)]
impl ToString for RibosomeErrorCode {
    fn to_string(&self) -> String {
        match self {
            RibosomeErrorCode::Unspecified                     => "Unspecified",
            RibosomeErrorCode::ArgumentDeserializationFailed   => "Argument deserialization failed",
            RibosomeErrorCode::OutOfMemory                     => "Out of memory",
            RibosomeErrorCode::ReceivedWrongActionResult       => "Received wrong action result",
            RibosomeErrorCode::CallbackFailed                  => "Callback failed",
            RibosomeErrorCode::RecursiveCallForbidden          => "Recursive call forbidden",
            RibosomeErrorCode::ResponseSerializationFailed     => "Response serialization failed",
            RibosomeErrorCode::NotAnAllocation                 => "Not an allocation",
            RibosomeErrorCode::ZeroSizedAllocation             => "Zero-sized allocation",
            RibosomeErrorCode::UnknownEntryType                => "Unknown entry type",
        }.to_string()
    }
}

impl TryFrom<RibosomeErrorCode> for JsonString {
    type Error = HolochainError;
    fn try_from(v: RibosomeErrorCode) -> JsonResult {
        default_try_to_json(v)
    }
}

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
        match s.as_ref() {
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
            _ => Err(HolochainError::ErrorGeneric(String::from(
                "Unknown RibosomeErrorCode",
            ))),
        }
    }
}

impl RibosomeReturnCode {
    pub fn from_error(err_code: RibosomeErrorCode) -> RibosomeReturnCode {
        RibosomeReturnCode::Failure(err_code)
    }

    pub fn from_offset(offset: u16) -> RibosomeReturnCode {
        match offset {
            0 => RibosomeReturnCode::Success,
            _ => RibosomeReturnCode::Failure(RibosomeErrorCode::from_offset(offset)),
        }
    }
}

impl RibosomeErrorCode {
    pub fn from_offset(offset: u16) -> Self {
        match offset {
            0 => unreachable!(),
            2 => RibosomeErrorCode::ArgumentDeserializationFailed,
            3 => RibosomeErrorCode::OutOfMemory,
            4 => RibosomeErrorCode::ReceivedWrongActionResult,
            5 => RibosomeErrorCode::CallbackFailed,
            6 => RibosomeErrorCode::RecursiveCallForbidden,
            7 => RibosomeErrorCode::ResponseSerializationFailed,
            8 => RibosomeErrorCode::NotAnAllocation,
            9 => RibosomeErrorCode::ZeroSizedAllocation,
            10 => RibosomeErrorCode::UnknownEntryType,
            1 | _ => RibosomeErrorCode::Unspecified,
        }
    }

    pub fn from_return_code(ret_code: RibosomeReturnCode) -> Self {
        match ret_code {
            RibosomeReturnCode::Success => unreachable!(),
            RibosomeReturnCode::Failure(rib_err) => rib_err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // a test function that returns our error result
    fn raises_holochain_error(yes: bool) -> Result<(), HolochainError> {
        if yes {
            Err(HolochainError::new("borked"))
        } else {
            Ok(())
        }
    }

    #[test]
    /// test that we can convert an error to a string
    fn to_string() {
        let err = HolochainError::new("foo");
        assert_eq!(r#"ErrorGeneric("foo")"#, err.to_string());
    }

    #[test]
    /// test that we can convert an error to valid JSON
    fn test_to_json() {
        let err = HolochainError::new("foo");
        assert_eq!(
            JsonString::from(r#"{"error":"foo"}"#),
            JsonString::from(err),
        );
    }

    #[test]
    /// smoke test new errors
    fn can_instantiate() {
        let err = HolochainError::new("borked");

        assert_eq!(HolochainError::ErrorGeneric("borked".to_string()), err);
    }

    #[test]
    /// test errors as a result and destructuring
    fn can_raise_holochain_error() {
        let err = raises_holochain_error(true).expect_err("should return an error when yes=true");

        match err {
            HolochainError::ErrorGeneric(msg) => assert_eq!(msg, "borked"),
            _ => panic!("raises_holochain_error should return an ErrorGeneric"),
        };
    }

    #[test]
    /// test errors as a returned result
    fn can_return_result() {
        let result = raises_holochain_error(false);

        assert!(result.is_ok());
    }

    #[test]
    /// show Error implementation for HolochainError
    fn error_test() {
        for (input, output) in vec![
            (HolochainError::ErrorGeneric(String::from("foo")), "foo"),
            (HolochainError::NotImplemented, "not implemented"),
            (HolochainError::LoggingError, "logging failed"),
            (HolochainError::DnaMissing, "DNA is missing"),
            (
                HolochainError::Dna(DnaError::ZomeNotFound(String::from("foo"))),
                "foo",
            ),
            (
                HolochainError::Dna(DnaError::CapabilityNotFound(String::from("foo"))),
                "foo",
            ),
            (
                HolochainError::Dna(DnaError::ZomeFunctionNotFound(String::from("foo"))),
                "foo",
            ),
            (HolochainError::IoError(String::from("foo")), "foo"),
            (
                HolochainError::SerializationError(String::from("foo")),
                "foo",
            ),
            (
                HolochainError::InvalidOperationOnSysEntry,
                "operation cannot be done on a system entry type",
            ),
            (
                HolochainError::DoesNotHaveCapabilityToken,
                "Caller does not have Capability to make that call",
            ),
        ] {
            assert_eq!(output, input.description());
        }
    }

    #[test]
    fn ribosome_return_code_round_trip() {
        let oom =
            RibosomeReturnCode::from_offset(((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16);
        assert_eq!(
            RibosomeReturnCode::Failure(RibosomeErrorCode::OutOfMemory),
            oom
        );
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
    fn ribosome_error_report_to_string() {
        let description = "This is a unit test error description";
        let report = RibosomeErrorReport {
            description: description.to_string(),
            file_name: file!().to_string(),
            line: line!().to_string(),
        };

        assert_ne!(
            report.to_string(),
            RibosomeErrorReport {
                description: description.to_string(),
                file_name: file!().to_string(),
                line: line!().to_string(),
            }.to_string(),
        );
    }
}
