use serde_json;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

/// track json serialization with the rust type system!
/// JsonString wraps a string containing JSON serialized data
/// avoid accidental double-serialization or forgetting to serialize
/// serialize any type consistently including hard-to-reach places like Option<Entry> and Result
/// JsonString must not itself be serialized/deserialized
/// instead, implement and use the native `From` trait to move between types
/// - moving to/from String, str, JsonString and JsonString simply (un)wraps it as raw JSON data
/// - moving to/from any other type must offer a reliable serialization/deserialization strategy
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct JsonString(String);

impl JsonString {
    /// represents None when implementing From<Option<Foo>>
    pub fn none() -> JsonString { JsonString::from("null") }
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

impl<'a> From<&'a JsonString> for String {
    fn from(json_string: &JsonString) -> String {
        String::from(json_string.to_owned())
    }
}

impl From<&'static str> for JsonString {
    fn from(s: &str) -> JsonString {
        JsonString::from(String::from(s))
    }
}

impl<S: Serialize> From<S> for JsonString {
    fn from(serializable: S) -> JsonString {
        JsonString::from(serde_json::to_string(&serializable).expect("could not serialize"))
    }
}

impl<T: Serialize, E: Serialize> From<Result<T, E>> for JsonString {
    fn from(result: Result<T, E>) -> JsonString {
        JsonString::from(serde_json::to_string(&result).expect("could not Json serialize result"))
    }
}

impl Display for JsonString {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            String::from(self),
        )
    }
}

/// generic type to facilitate Jsonifying strings
/// JsonString simply wraps String and str as-is but will Jsonify RawString("foo") as "\"foo\""
pub struct RawString(String);

impl From<&'static str> for RawString {
    fn from(s: &str) -> RawString {
        RawString(s.to_owned())
    }
}

impl From<String> for RawString {
    fn from(s: String) -> RawString {
        RawString(s)
    }
}

impl From<RawString> for JsonString {
    fn from(raw_string: RawString) -> JsonString {
        JsonString::from(serde_json::to_string(&raw_string.0).expect("could not Jsonify RawString"))
    }
}
