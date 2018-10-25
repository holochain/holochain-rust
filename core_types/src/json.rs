use serde::Serialize;
use serde_json;
use std::fmt::{Display, Formatter, Result as FmtResult};

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
    pub fn none() -> JsonString {
        JsonString::from("null")
    }

    /// achieves the same outcome as serde_json::to_vec()
    pub fn into_bytes(&self) -> Vec<u8> {
        self.0.to_owned().into_bytes()
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

impl<T: Serialize> From<Vec<T>> for JsonString {
    fn from(vector: Vec<T>) -> JsonString {
        JsonString::from(serde_json::to_string(&vector).expect("could not Jsonify vector"))
    }
}

impl<T: Into<JsonString>, E: Into<JsonString>> From<Result<T, E>> for JsonString {
    fn from(result: Result<T, E>) -> JsonString {
        JsonString::from(
            serde_json::to_string(&match result {
                Ok(t) => {
                    let json_string: JsonString = t.into();
                    Ok(String::from(json_string))
                }
                Err(e) => {
                    let json_string: JsonString = e.into();
                    Err(String::from(json_string))
                }
            }).expect("could not serialize Result"),
        )
    }
}

impl Display for JsonString {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", String::from(self),)
    }
}

/// generic type to facilitate Jsonifying values directly
/// JsonString simply wraps String and str as-is but will Jsonify RawString("foo") as "\"foo\""
/// RawString must not implement Serialize and Deserialize itself
#[derive(PartialEq, Debug)]
pub struct RawString(serde_json::Value);

impl From<&'static str> for RawString {
    fn from(s: &str) -> RawString {
        RawString(serde_json::Value::String(s.to_owned()))
    }
}

impl From<String> for RawString {
    fn from(s: String) -> RawString {
        RawString(serde_json::Value::String(s))
    }
}

impl From<f64> for RawString {
    fn from(i: f64) -> RawString {
        RawString(serde_json::Value::Number(
            serde_json::Number::from_f64(i).expect("could not accept number"),
        ))
    }
}

impl From<i32> for RawString {
    fn from(i: i32) -> RawString {
        RawString::from(i as f64)
    }
}

impl From<RawString> for String {
    fn from(raw_string: RawString) -> String {
        // this will panic if RawString does not contain a string!
        // use JsonString::from(...) to stringify numbers or other values
        // @see raw_from_number_test()
        String::from(
            raw_string
                .0
                .as_str()
                .expect("could not extract inner string for RawString"),
        )
    }
}

impl From<RawString> for JsonString {
    fn from(raw_string: RawString) -> JsonString {
        JsonString::from(serde_json::to_string(&raw_string.0).expect("could not Jsonify RawString"))
    }
}

impl From<JsonString> for RawString {
    fn from(json_string: JsonString) -> RawString {
        RawString(
            serde_json::from_str(&String::from(json_string.clone())).expect(&format!(
                "could not deserialize JsonString: {:?}",
                json_string
            )),
        )
    }
}

#[cfg(test)]
pub mod tests {
    use json::{JsonString, RawString};

    #[test]
    fn json_none_test() {
        assert_eq!(String::from("null"), String::from(JsonString::none()),);
    }

    #[test]
    fn json_into_bytes_test() {
        assert_eq!(JsonString::from("foo").into_bytes(), vec![102, 111, 111],);
    }

    #[test]
    /// show From<&str> and From<String> for JsonString
    fn json_from_string_test() {
        assert_eq!(String::from("foo"), String::from(JsonString::from("foo")),);

        assert_eq!(
            String::from("foo"),
            String::from(JsonString::from(String::from("foo"))),
        );

        assert_eq!(String::from("foo"), String::from(&JsonString::from("foo")),);
    }

    #[test]
    /// show From<Vec<T>> for JsonString
    fn json_from_vec() {
        assert_eq!(
            String::from("[\"foo\",\"bar\"]"),
            String::from(JsonString::from(vec!["foo", "bar"])),
        );
    }

    #[test]
    /// show From<&str> and From<String> for RawString
    fn raw_from_string_test() {
        assert_eq!(RawString::from(String::from("foo")), RawString::from("foo"),);
    }

    #[test]
    /// show From<RawString> for String
    fn string_from_raw_test() {
        assert_eq!(String::from("foo"), String::from(RawString::from("foo")),);
    }

    #[test]
    /// show From<RawString> for JsonString
    fn json_from_raw_test() {
        assert_eq!(
            String::from("\"foo\""),
            String::from(JsonString::from(RawString::from("foo"))),
        );
    }

    #[test]
    /// show From<JsonString> for RawString
    fn raw_from_json_test() {
        assert_eq!(
            String::from(RawString::from(JsonString::from("\"foo\""))),
            String::from("foo"),
        );
    }

    #[test]
    /// show From<number> for RawString
    fn raw_from_number_test() {
        assert_eq!(
            String::from("1.0"),
            String::from(JsonString::from(RawString::from(1))),
        );
    }
}
