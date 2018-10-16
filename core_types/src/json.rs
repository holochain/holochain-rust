use std::fmt::{Display, Formatter, Result as FmtResult};
use serde_json;
use serde::Serialize;

#[derive(Debug, PartialEq)]
pub struct JsonString(String);

impl JsonString {
    pub fn none() -> JsonString {
        JsonString::from("null")
    }
}

impl From<String> for JsonString {
    fn from(s: String) -> JsonString {
        JsonString(s)
    }
}

impl From<JsonString> for String {
    fn from(json_string: JsonString) -> String {
        json_string.0
    }
}

impl From<&'static str> for JsonString {
    fn from(s: &str) -> JsonString {
        JsonString::from(String::from(s))
    }
}

impl<T: Serialize, E: Serialize> From<Result<T, E>> for JsonString {
    fn from(result: Result<T, E>) -> JsonString {
        JsonString::from(serde_json::to_string(&result).expect("could not Json serialize result"))
    }
}

impl Display for JsonString {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
