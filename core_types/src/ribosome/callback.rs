use entry::{SerializedEntry};
use error::{RibosomeReturnCode};
use json::*;
use serde_json;
use validation::ValidationPackageDefinition;

#[derive(Debug, Serialize)]
pub enum CallbackParams {
    Genesis,
    ValidateCommit(SerializedEntry),
    // @TODO call this from somewhere
    // @see https://github.com/holochain/holochain-rust/issues/201
    Receive,
}

impl ToString for CallbackParams {
    fn to_string(&self) -> String {
        match self {
            CallbackParams::Genesis => String::new(),
            CallbackParams::ValidateCommit(serialized_entry) => {
                String::from(JsonString::from(serialized_entry.to_owned()))
            }
            CallbackParams::Receive => String::new(),
        }
    }
}

impl From<CallbackParams> for JsonString {
    fn from(v: CallbackParams) -> Self {
        default_to_json(v)
    }
}
// @TODO
// no idea why this is needed!
impl<'a> From<&'a CallbackParams> for JsonString {
    fn from(v: &CallbackParams) -> Self {
        default_to_json(v.to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CallbackResult {
    Pass,
    Fail(String),
    NotImplemented,
    ValidationPackageDefinition(ValidationPackageDefinition),
}

impl From<CallbackResult> for JsonString {
    fn from(v: CallbackResult) -> Self {
        default_to_json(v)
    }
}

impl From<JsonString> for CallbackResult {
    fn from(json_string: JsonString) -> CallbackResult {
        let try: Result<CallbackResult, serde_json::Error> =
            serde_json::from_str(&String::from(json_string.clone()));
        match try {
            Ok(callback_result) => callback_result,
            Err(_) => CallbackResult::Fail(String::from(json_string)),
        }
    }
}

impl From<RibosomeReturnCode> for CallbackResult {
    fn from(ribosome_return_code: RibosomeReturnCode) -> CallbackResult {
        match ribosome_return_code {
            RibosomeReturnCode::Failure(ribosome_error_code) => {
                CallbackResult::Fail(ribosome_error_code.to_string())
            }
            RibosomeReturnCode::Success => CallbackResult::Pass,
        }
    }
}
