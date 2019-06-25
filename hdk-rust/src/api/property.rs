use super::super::PROPERTIES;
use error::{ZomeApiError, ZomeApiResult};
use holochain_json_api::json::JsonString;
use serde_json::Value;

// Returns a DNA property, which are defined by the DNA developer.
// They are custom values that are defined in the DNA file
// that can be used in the zome code for defining configurable behaviors.
// (e.g. Name, Language, Description, Author, etc.).
pub fn property<S: Into<String>>(name: S) -> ZomeApiResult<JsonString> {
    let properties: Value = serde_json::from_str(&PROPERTIES.to_string()).map_err(|_| {
        ZomeApiError::from("DNA Properties could not be parsed as JSON".to_string())
    })?;

    properties
        .get(name.into())
        .map(|value| JsonString::from(value.clone()))
        .ok_or_else(|| ZomeApiError::from("field does not exist in DNA properties".to_string()))
}
