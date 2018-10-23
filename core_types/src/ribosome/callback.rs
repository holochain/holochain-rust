use entry::Entry;
use entry::SerializedEntry;
use json::JsonString;
use serde_json;

#[derive(Debug)]
pub enum CallbackParams {
    Genesis,
    ValidateCommit(Entry),
    // @TODO call this from somewhere
    // @see https://github.com/holochain/holochain-rust/issues/201
    Receive,
}

impl ToString for CallbackParams {
    fn to_string(&self) -> String {
        match self {
            CallbackParams::Genesis => String::new(),
            CallbackParams::ValidateCommit(entry) => {
                String::from(JsonString::from(SerializedEntry::from(entry.to_owned())))
            }
            CallbackParams::Receive => String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CallbackResult {
    Pass,
    Fail(String),
    NotImplemented,
}

impl From<JsonString> for CallbackResult {
    fn from(json_string: JsonString) -> CallbackResult {
        serde_json::from_str(&String::from(json_string.clone()))
            .expect(&format!("could not deserialize CallbackResult: {:?}", json_string))
    }
}

impl From<CallbackResult> for JsonString {
    fn from(callback_result: CallbackResult) -> JsonString {
        JsonString::from(serde_json::to_string(&callback_result)
            .expect("could not Jsonify CallbackResult"))
    }
}
