use self::HolochainError::*;
use std::{error::Error, fmt};
use walkdir::Error as WalkdirError;
use std::path::Path;
use std::io;
use std::io::Error as IoError;
use json::ToJson;

/// module for holding Holochain specific errors

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum HolochainError {
    ErrorGeneric(String),
    InstanceNotActive,
    InstanceActive,
    NotImplemented,
    LoggingError,
    DnaMissing,
    ZomeNotFound(String),
    CapabilityNotFound(String),
    ZomeFunctionNotFound(String),
    IoError(String),
    SerializationError(String),
}

impl HolochainError {
    pub fn new(msg: &str) -> HolochainError {
        HolochainError::ErrorGeneric(msg.to_string())
    }
}

impl ToJson for HolochainError {
    /// @TODO round trip this
    /// @see https://github.com/holochain/holochain-rust/issues/193
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(format!("{{\"error\":\"{}\"}}", self.description()))
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
            InstanceNotActive => "the instance is not active",
            InstanceActive => "the instance is active",
            LoggingError => "logging failed",
            DnaMissing => "DNA is missing",
            ZomeNotFound(err_msg) => &err_msg,
            CapabilityNotFound(err_msg) => &err_msg,
            ZomeFunctionNotFound(err_msg) => &err_msg,
            IoError(err_msg) => &err_msg,
        }
    }
}

fn reason_for_io_error(error: &IoError) -> String {
    match error.kind() {
        io::ErrorKind::InvalidData => format!("contains invalid data: {}", error),
        io::ErrorKind::PermissionDenied => format!("missing permissions to read: {}", error),
        _ => format!("unexpected error: {}", error),
    }
}

impl From<WalkdirError> for HolochainError {
    fn from(error: WalkdirError) -> Self {
        // adapted from https://docs.rs/walkdir/2.2.5/walkdir/struct.Error.html#example
        let path = error.path().unwrap_or(Path::new("")).display();
        let reason = match error.io_error() {
            Some(inner) => reason_for_io_error(inner),
            None => String::new(),
        };
        HolochainError::IoError(
            format!("error at path: {}, reason: {}", path, reason)
        )
    }
}

impl From<IoError> for HolochainError {
    fn from(error: IoError) -> Self {
        HolochainError::IoError(reason_for_io_error(&error))
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
        assert_eq!(r#"{"error":"foo"}"#, err.to_json());
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
}
