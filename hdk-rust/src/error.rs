use self::ZomeApiError::*;
use holochain_core_types::json::JsonString;
use std::{error::Error, fmt};
use holochain_core_types::error::RibosomeErrorCode;
use std::convert::TryFrom;

pub type ZomeApiResult<T> = Result<T, ZomeApiError>;
#[derive(Deserialize, Default, Debug)]
pub struct ZomeApiInternalResult<T> {
    pub ok: bool,
    pub value: T,
    pub error: String,
}

impl<T: TryFrom<JsonString>> From<ZomeApiInternalResult<T>> for Result<T, ZomeApiError> {
    fn from(result: ZomeApiInternalResult<T>) -> Self {
        if result.ok {
            Ok(T::try_from(JsonString::from(result.value))?)
        } else {
            Err(ZomeApiError::from(result.error))
        }
    }
}

/// Error for DNA developers to use in their zome code.
/// They do not have to send this error back to Ribosome unless its an InternalError.
#[derive(Debug, Serialize)]
pub enum ZomeApiError {
    Internal(String),
    FunctionNotImplemented,
    HashNotFound,
    ValidationFailed(String),
}

impl From<ZomeApiError> for JsonString {
    fn from(zome_api_error: ZomeApiError) -> JsonString {
        JsonString::from(json!({ "error": zome_api_error }))
    }
}

impl From<String> for ZomeApiError {
    fn from(s: String) -> ZomeApiError {
        ZomeApiError::Internal(s)
    }
}

impl From<RibosomeErrorCode> for ZomeApiError {
    fn from(ribosome_error_code: RibosomeErrorCode) -> ZomeApiError {
        ZomeApiError::from(ribosome_error_code.to_string())
    }
}

impl Error for ZomeApiError {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn description(&self) -> &str {
        match self {
            Internal(msg)           => &msg,
            FunctionNotImplemented  => "Function not implemented",
            HashNotFound            => "Hash not found",
            ValidationFailed(msg)   => &msg,
        }
    }
}

impl fmt::Display for ZomeApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO seems weird to use debug for display
        // replacing {:?} with {} gives a stack overflow on to_string() (there's a test for this)
        // what is the right way to do this?
        // @see https://github.com/holochain/holochain-rust/issues/223
        write!(f, "{:?}", self)
    }
}
