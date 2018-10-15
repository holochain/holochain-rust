pub struct JsonString(String);

impl From<String> for JsonString {
    fn from(s: String) -> JsonString {
        JsonString(s)
    }
}
