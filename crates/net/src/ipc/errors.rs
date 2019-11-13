//! This module holds net_ipc custom error types.

use failure;
use std;

/// net_ipc-specific error types
#[derive(Debug, Clone, Fail)]
pub enum IpcError {
    /// Translate an Option<_> unwrap into a Result::Err
    #[fail(display = "MissingDataError")]
    MissingDataError,

    /// Socket timeout
    #[fail(display = "Timeout")]
    Timeout,

    /// Otherwise undefined error message
    #[fail(display = "IpcError: {}", error)]
    GenericError { error: String },
}

/// Macro akin to `bail!()` but returns an IpcError::GenericError.
#[macro_export]
macro_rules! bail_generic {
    ($e:expr) => {
        return Err(IpcError::GenericError {
            error: $e.to_string(),
        }.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(IpcError::GenericError {
            error: format!($fmt, $($arg)+),
        }.into());
    };
}

/// Default result type for net_ipc modules that `use errors::*`.
pub type Result<T> = std::result::Result<T, failure::Error>;
