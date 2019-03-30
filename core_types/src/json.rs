//! The JsonString type is defined here. It is used throughout Holochain
//! to enforce a standardized serialization of data to/from json.

use crate::error::{HcResult, HolochainError};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::{
    convert::TryFrom,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

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
    /// a null JSON value
    /// e.g. represents None when implementing From<Option<Foo>>
    pub fn null() -> JsonString {
        JsonString::from_json("null")
    }

    pub fn empty_object() -> JsonString {
        JsonString::from_json("{}")
    }

    pub fn is_null(&self) -> bool {
        self == &Self::null()
    }

    /// achieves the same outcome as serde_json::to_vec()
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    // Creates a JsonString from stringified json
    // replaces From<String> for JsonString and allows
    // for implementation of TryFrom<JsonString> on generic types
    pub fn from_json(s: &str) -> JsonString {
        let cleaned = s
            // remove whitespace from both ends
            .trim()
            // remove null characters from both endsi
            .trim_matches(char::from(0));
        JsonString(cleaned.to_owned())        
    }
}

impl From<String> for JsonString {
    fn from(s: String) -> JsonString {
        let cleaned = s
            // remove whitespace from both ends
            .trim()
            // remove null characters from both ends
            .trim_matches(char::from(0));
        JsonString(cleaned.to_owned())
    }
}

impl From<bool> for JsonString {
    fn from(u: bool) -> JsonString {
        default_to_json(u)
    }
}

impl From<u32> for JsonString {
    fn from(u: u32) -> JsonString {
        default_to_json(u)
    }
}

impl From<i32> for JsonString {
    fn from(u: i32) -> JsonString {
        default_to_json(u)
    }
}

impl From<u64> for JsonString {
    fn from(u: u64) -> JsonString {
        default_to_json(u)
    }
}

impl From<u128> for JsonString {
    fn from(u: u128) -> JsonString {
        default_to_json(u)
    }
}

impl TryFrom<JsonString> for bool {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl TryFrom<JsonString> for u32 {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl TryFrom<JsonString> for u64 {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl From<serde_json::Value> for JsonString {
    fn from(v: serde_json::Value) -> JsonString {
        JsonString::from_json(&v.to_string())
    }
}

impl From<JsonString> for String {
    fn from(json_string: JsonString) -> String {
        json_string.0
    }
}

impl<'a> From<&'a JsonString> for &'a str {
    fn from(json_string: &'a JsonString) -> &'a str {
        &json_string.0
    }
}

impl<'a> From<&'a JsonString> for String {
    fn from(json_string: &JsonString) -> String {
        String::from(json_string.to_owned())
    }
}

impl From<&'static str> for JsonString {
    fn from(s: &str) -> JsonString {
        JsonString::from_json(&String::from(s))
    }
}

impl<T: Serialize> From<Vec<T>> for JsonString {
    fn from(vector: Vec<T>) -> JsonString {
        JsonString::from_json(&serde_json::to_string(&vector).expect("could not Jsonify vector"))
    }
}

/// signifies type can be converted to JsonString in Err from some Result
/// can't use std::error::Error for this because String has Error as a reserved future trait
pub trait JsonError {}

impl JsonError for HolochainError {}

impl<T: Into<JsonString>, E: Into<JsonString> + JsonError> From<Result<T, E>> for JsonString {
    fn from(result: Result<T, E>) -> JsonString {
        let is_ok = result.is_ok();
        let inner_json: JsonString = match result {
            Ok(inner) => inner.into(),
            Err(inner) => inner.into(),
        };
        let inner_string = String::from(inner_json);
        JsonString::from_json(&format!(
            "{{\"{}\":{}}}",
            if is_ok { "Ok" } else { "Err" },
            inner_string
        ))
    }
}

impl<T: Into<JsonString>> From<Result<T, String>> for JsonString {
    fn from(result: Result<T, String>) -> JsonString {
        let is_ok = result.is_ok();
        let inner_json: JsonString = match result {
            Ok(inner) => inner.into(),
            // strings need this special handling c.f. Error
            Err(inner) => RawString::from(inner).into(),
        };
        let inner_string = String::from(inner_json);
        JsonString::from_json(&format!(
            "{{\"{}\":{}}}",
            if is_ok { "Ok" } else { "Err" },
            inner_string
        ))
    }
}

pub type JsonResult = Result<JsonString, HolochainError>;

impl From<()> for JsonString {
    fn from(_: ()) -> Self {
        default_to_json(())
    }
}

impl TryFrom<JsonString> for () {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl Display for JsonString {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", String::from(self),)
    }
}

/// if all you want to do is implement the default behaviour then use #[derive(DefaultJson)]
/// should only be used with From<S> for JsonString
/// i.e. when failure should be impossible so an expect is ok
/// this is always true for serializable structs/enums
/// standard boilerplate:
/// impl From<MyStruct> for JsonString {
///     fn from(v: MyStruct) -> Self {
///         default_to_json(v)
///     }
/// }
pub fn default_to_json<V: Serialize + Debug>(v: V) -> JsonString {
    serde_json::to_string(&v)
        .map(|s| JsonString::from_json(&s))
        .map_err(|e| HolochainError::SerializationError(e.to_string()))
        .unwrap_or_else(|_| panic!("could not Jsonify: {:?}", v))
}

/// if all you want to do is implement the default behaviour then use #[derive(DefaultJson)]
/// standard boilerplate should include HolochainError as the Error:
/// impl TryFrom<JsonString> for T {
///     type Error = HolochainError;
///     fn try_from(j: JsonString) -> HcResult<Self> {
///         default_try_from_json(j)
///     }
/// }
pub fn default_try_from_json<D: DeserializeOwned>(
    json_string: JsonString,
) -> Result<D, HolochainError> {
    serde_json::from_str(&String::from(&json_string))
        .map_err(|e| HolochainError::SerializationError(e.to_string()))
}

pub trait DefaultJson:
    Serialize + DeserializeOwned + TryFrom<JsonString> + Into<JsonString>
{
}

/// generic type to facilitate Jsonifying values directly
/// JsonString simply wraps String and str as-is but will Jsonify RawString("foo") as "\"foo\""
/// RawString must not implement Serialize because it should always convert to JsonString with from
/// RawString can implement Deserialize because JsonString uses default serde to step down
#[derive(PartialEq, Debug, Clone, Deserialize)]
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
        RawString::from(f64::from(i))
    }
}

impl From<RawString> for String {
    fn from(raw_string: RawString) -> String {
        // this will panic if RawString does not contain a string!
        // use JsonString::from(...) to stringify numbers or other values
        // @see raw_from_number_test()
        String::from(raw_string.0.as_str().unwrap_or_else(|| {
            panic!(
                "could not extract inner string for RawString: {:?}",
                &raw_string
            )
        }))
    }
}

/// it should always be possible to Jsonify RawString, if not something is very wrong
impl From<RawString> for JsonString {
    fn from(raw_string: RawString) -> JsonString {
        JsonString::from_json(
            &serde_json::to_string(&raw_string.0)
                .unwrap_or_else(|_| panic!("could not Jsonify RawString: {:?}", &raw_string)),
        )
    }
}

/// converting a JsonString to RawString can fail if the JsonString is not a serialized string
impl TryFrom<JsonString> for RawString {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> HcResult<Self> {
        default_try_from_json(j)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        error::HolochainError,
        json::{JsonString, RawString},
    };
    use serde_json;
    use std::convert::TryFrom;

    #[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq, Clone)]
    struct DeriveTest {
        foo: String,
    }

    #[test]
    fn default_json_round_trip_test() {
        let test = DeriveTest { foo: "bar".into() };
        let expected = JsonString::from_json("{\"foo\":\"bar\"}");
        assert_eq!(expected, JsonString::from(test.clone()),);

        assert_eq!(&DeriveTest::try_from(expected).unwrap(), &test,);

        assert_eq!(
            test.clone(),
            DeriveTest::try_from(JsonString::from(test)).unwrap(),
        );
    }

    #[test]
    fn json_none_test() {
        assert_eq!(String::from("null"), String::from(JsonString::null()),);
    }

    #[test]
    fn json_into_bytes_test() {
        // note that the byte array has the quote character '/"' at the beginnging and end so it is actually valid json
        assert_eq!(JsonString::from(RawString::from("foo")).to_bytes(), vec![34, 102, 111, 111, 34],);
    }

    #[test]
    fn json_result_round_trip_test() {
        let result: Result<String, HolochainError> =
            Err(HolochainError::ErrorGeneric("foo".into()));

        assert_eq!(
            JsonString::from(result),
            JsonString::from_json("{\"Err\":{\"ErrorGeneric\":\"foo\"}}"),
        );

        let result: Result<String, String> = Err(String::from("foo"));

        assert_eq!(
            JsonString::from(result),
            JsonString::from_json("{\"Err\":\"foo\"}"),
        )
    }

    #[test]
    /// show From<&str> and From<String> for JsonString
    fn json_from_string_test() {
        assert_eq!(
            String::from("\"foo\""),
            String::from(JsonString::from(RawString::from("foo"))),
        );

        assert_eq!(
            String::from("\"foo\""),
            String::from(JsonString::from_json(&String::from("\"foo\""))),
        );

        assert_eq!(
            String::from("\"foo\""),
            String::from(&JsonString::from(RawString::from("foo"))),
        );
    }

    #[test]
    /// show From<serde_json::Value> for JsonString
    fn json_from_serde_test() {
        assert_eq!(
            String::from("\"foo\""),
            String::from(JsonString::from(serde_json::Value::from("foo"))),
        );
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
            String::from(RawString::try_from(JsonString::from("\"foo\"")).unwrap()),
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
