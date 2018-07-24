use ::std;
use failure;

#[derive(Debug, Fail)]
pub enum IpcError {
    #[fail(display = "NoneError")]
    NoneError,
    #[fail(display = "Timeout")]
    Timeout,
    #[fail(display = "IpcError: {}", error)]
    GenericError { error: String },
}

#[macro_export]
macro_rules! gerr {
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

pub type Result<T> = std::result::Result<T, failure::Error>;
