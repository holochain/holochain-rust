//! This module contains Error type definitions that are used throughout Holochain, and the Ribosome in particular,
//! which is responsible for mounting and running instances of DNA, and executing WASM code.

mod dna_error;
mod ribosome_error;

pub use self::{dna_error::*, ribosome_error::*};
use sync::HcLockError;

use self::HolochainError::*;
use futures::channel::oneshot::Canceled as FutureCanceled;
use holochain_json_api::{
    error::{JsonError, JsonResult},
    json::*,
};
use holochain_persistence_api::{error::PersistenceError, hash::HashString};
use lib3h_crypto_api::CryptoError;

use serde_json::Error as SerdeError;
use std::{
    error::Error,
    fmt,
    io::{self, Error as IoError},
    option::NoneError,
};

//--------------------------------------------------------------------------------------------------
// CoreError
//--------------------------------------------------------------------------------------------------

/// Holochain Core Error struct
/// Any Error in Core should be wrapped in a CoreError so it can be passed to the Zome
/// and back to the Holochain Instance via wasm memory.
/// Follows the Error + ErrorKind pattern
/// Holds extra debugging info for indicating where in code ther error occured.
#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson, PartialEq, Eq, Hash)]
pub struct CoreError {
    pub kind: HolochainError,
    pub file: String,
    pub line: String,
    // TODO #395 - Add advance error debugging info
    // pub stack_trace: Backtrace
}

// Error trait by using the inner Error
impl Error for CoreError {
    fn cause(&self) -> Option<&dyn Error> {
        self.kind.source()
    }
}

impl CoreError {
    pub fn new(hc_err: HolochainError) -> Self {
        CoreError {
            kind: hc_err,
            file: String::new(),
            line: String::new(),
        }
    }
}

impl ::std::convert::TryFrom<ZomeApiInternalResult> for CoreError {
    type Error = HolochainError;
    fn try_from(zome_api_internal_result: ZomeApiInternalResult) -> Result<Self, Self::Error> {
        if zome_api_internal_result.ok {
            Err(HolochainError::ErrorGeneric(
                "Attempted to deserialize CoreError from a non-error ZomeApiInternalResult".into(),
            ))
        } else {
            let hc_error: JsonString = JsonString::from_json(&zome_api_internal_result.error);
            let ce: JsonResult<_> = CoreError::try_from(hc_error);
            ce.map_err(|err: JsonError| {
                let hc_error: HolochainError = err.into();
                hc_error
            })
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Holochain Core error: {}\n  --> {}:{}\n",
            self.kind, self.file, self.line,
        )
    }
}

//--------------------------------------------------------------------------------------------------
// HolochainError
//--------------------------------------------------------------------------------------------------

/// TODO rename to CoreErrorKind
/// Enum holding all Holochain Core errors
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DefaultJson, Hash, PartialOrd, Ord,
)]
pub enum HolochainError {
    ErrorGeneric(String),
    CryptoError(CryptoError),
    NotImplemented(String),
    LoggingError,
    DnaMissing,
    Dna(DnaError),
    IoError(String),
    SerializationError(String),
    InvalidOperationOnSysEntry,
    CapabilityCheckFailed,
    ValidationFailed(String),
    ValidationPending,
    Ribosome(RibosomeErrorCode),
    RibosomeFailed(String),
    ConfigError(String),
    Timeout,
    InitializationFailed(String),
    LifecycleError(String),
    DnaHashMismatch(HashString, HashString),
    EntryNotFoundLocally,
    EntryIsPrivate,
    List(Vec<HolochainError>),
}

pub type HcResult<T> = Result<T, HolochainError>;

impl HolochainError {
    pub fn new(msg: &str) -> HolochainError {
        HolochainError::ErrorGeneric(msg.to_string())
    }
}

impl From<rust_base58::base58::FromBase58Error> for HolochainError {
    fn from(e: rust_base58::base58::FromBase58Error) -> Self {
        HolochainError::SerializationError(format!("{}", e))
    }
}

impl fmt::Display for HolochainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorGeneric(err_msg) => write!(f, "{}", err_msg),
            CryptoError(crypto_error) => write!(f, "{}", crypto_error),
            NotImplemented(description) => write!(f, "not implemented: {}", description),
            LoggingError => write!(f, "logging failed"),
            DnaMissing => write!(f, "DNA is missing"),
            Dna(dna_err) => write!(f, "{}", dna_err),
            IoError(err_msg) => write!(f, "{}", err_msg),
            SerializationError(err_msg) => write!(f, "{}", err_msg),
            InvalidOperationOnSysEntry => {
                write!(f, "operation cannot be done on a system entry type")
            }
            CapabilityCheckFailed => write!(f, "Caller does not have Capability to make that call"),
            ValidationFailed(fail_msg) => write!(f, "{}", fail_msg),
            ValidationPending => write!(f, "Entry validation could not be completed"),
            Ribosome(err_code) => write!(f, "{}", err_code.as_str()),
            RibosomeFailed(fail_msg) => write!(f, "{}", fail_msg),
            ConfigError(err_msg) => write!(f, "{}", err_msg),
            Timeout => write!(f, "timeout"),
            InitializationFailed(err_msg) => write!(f, "{}", err_msg),
            LifecycleError(err_msg) => write!(f, "{}", err_msg),
            DnaHashMismatch(hash1, hash2) => write!(
                f,
                "Provided DNA hash does not match actual DNA hash! {} != {}",
                hash1, hash2
            ),
            EntryNotFoundLocally => write!(f, "The requested entry could not be found locally"),
            EntryIsPrivate => write!(
                f,
                "The requested entry is private and should not be shared via gossip"
            ),
            List(list) => {
                //most windows system know that \n is a newline so we should be good.
                let error_list = list
                    .iter()
                    .map(|s| format!("{}", s))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "A list of errors has been generated {}", error_list)
            }
        }
    }
}

impl Error for HolochainError {}

impl From<HolochainError> for String {
    fn from(holochain_error: HolochainError) -> Self {
        holochain_error.to_string()
    }
}

impl From<PersistenceError> for HolochainError {
    fn from(persistence_error: PersistenceError) -> Self {
        match persistence_error {
            PersistenceError::ErrorGeneric(e) => HolochainError::ErrorGeneric(e),
            PersistenceError::SerializationError(e) => HolochainError::SerializationError(e),
            PersistenceError::IoError(e) => HolochainError::IoError(e),
        }
    }
}

impl From<JsonError> for HolochainError {
    fn from(json_error: JsonError) -> Self {
        match json_error {
            JsonError::ErrorGeneric(e) => HolochainError::ErrorGeneric(e),
            JsonError::SerializationError(e) => HolochainError::SerializationError(e),
            JsonError::IoError(e) => HolochainError::IoError(e),
        }
    }
}

impl From<String> for HolochainError {
    fn from(error: String) -> Self {
        HolochainError::new(&error)
    }
}

impl From<&'static str> for HolochainError {
    fn from(error: &str) -> Self {
        HolochainError::new(error)
    }
}

impl From<CryptoError> for HolochainError {
    fn from(error: CryptoError) -> Self {
        HolochainError::CryptoError(error)
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

impl<T> From<::std::sync::PoisonError<T>> for HolochainError {
    fn from(error: ::std::sync::PoisonError<T>) -> Self {
        HolochainError::ErrorGeneric(format!("sync poison error: {}", error))
    }
}

impl From<HcLockError> for HolochainError {
    fn from(error: HcLockError) -> Self {
        HolochainError::ErrorGeneric(format!("HcLockError: {:?}", error))
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

impl From<base64::DecodeError> for HolochainError {
    fn from(error: base64::DecodeError) -> Self {
        HolochainError::ErrorGeneric(format!("base64 decode error: {}", error.to_string()))
    }
}

impl From<std::str::Utf8Error> for HolochainError {
    fn from(error: std::str::Utf8Error) -> Self {
        HolochainError::ErrorGeneric(format!("std::str::Utf8Error error: {}", error.to_string()))
    }
}

impl From<FutureCanceled> for HolochainError {
    fn from(_: FutureCanceled) -> Self {
        HolochainError::ErrorGeneric("Failed future".to_string())
    }
}

impl From<NoneError> for HolochainError {
    fn from(_: NoneError) -> Self {
        HolochainError::ErrorGeneric("Expected Some and got None".to_string())
    }
}

impl From<hcid::HcidError> for HolochainError {
    fn from(error: hcid::HcidError) -> Self {
        HolochainError::ErrorGeneric(format!("{:?}", error))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, DefaultJson)]
pub struct ZomeApiInternalResult {
    pub ok: bool,
    pub value: String,
    pub error: String,
}

impl ZomeApiInternalResult {
    pub fn success<J: Into<JsonString>>(value: J) -> ZomeApiInternalResult {
        let json_string: JsonString = value.into();
        ZomeApiInternalResult {
            ok: true,
            value: json_string.into(),
            error: JsonString::null().into(),
        }
    }

    pub fn failure<J: Into<JsonString>>(value: J) -> ZomeApiInternalResult {
        let json_string: JsonString = value.into();
        ZomeApiInternalResult {
            ok: false,
            value: JsonString::null().into(),
            error: json_string.into(),
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
        assert_eq!("foo", err.to_string());
    }

    #[test]
    /// test that we can convert an error to valid JSON
    fn test_to_json() {
        let err = HolochainError::new("foo");
        assert_eq!(
            JsonString::from_json("{\"ErrorGeneric\":\"foo\"}"),
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
            (
                HolochainError::NotImplemented("reason".into()),
                "not implemented: reason",
            ),
            (HolochainError::LoggingError, "logging failed"),
            (HolochainError::DnaMissing, "DNA is missing"),
            (HolochainError::ConfigError(String::from("foo")), "foo"),
            (
                HolochainError::Dna(DnaError::ZomeNotFound(String::from("foo"))),
                "foo",
            ),
            (
                HolochainError::Dna(DnaError::TraitNotFound(String::from("foo"))),
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
                HolochainError::CapabilityCheckFailed,
                "Caller does not have Capability to make that call",
            ),
            (HolochainError::Timeout, "timeout"),
            (
                HolochainError::ValidationPending,
                "Entry validation could not be completed",
            ),
            (
                HolochainError::EntryNotFoundLocally,
                "The requested entry could not be found locally",
            ),
            (
                HolochainError::EntryIsPrivate,
                "The requested entry is private and should not be shared via gossip",
            ),
        ] {
            assert_eq!(output, &input.to_string());
        }
    }

    #[test]
    fn core_error_to_string() {
        let error =
            HolochainError::ErrorGeneric("This is a unit test error description".to_string());
        let report = CoreError {
            kind: error.clone(),
            file: file!().to_string(),
            line: line!().to_string(),
        };

        assert_ne!(
            report.to_string(),
            CoreError {
                kind: error,
                file: file!().to_string(),
                line: line!().to_string(),
            }
            .to_string(),
        );
    }

}
