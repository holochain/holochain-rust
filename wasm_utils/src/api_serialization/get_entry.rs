use holochain_core_types::error::HolochainError;
use holochain_core_types::error::HcResult;
use holochain_core_types::{entry::SerializedEntry, json::*};
use serde_json;
use std::convert::TryFrom;

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResultStatus {
    Found,
    NotFound,
}

// empty for now, need to implement get options
#[derive(Deserialize, Debug, Serialize)]
pub struct GetEntryOptions {}

#[derive(Deserialize, Debug, Serialize)]
pub struct GetEntryResult {
    pub status: GetResultStatus,
    pub maybe_serialized_entry: Option<SerializedEntry>,
}

impl GetEntryResult {
    pub fn found(serialized_entry: SerializedEntry) -> GetEntryResult {
        GetEntryResult {
            status: GetResultStatus::Found,
            maybe_serialized_entry: Some(serialized_entry),
        }
    }

    pub fn not_found() -> GetEntryResult {
        GetEntryResult {
            status: GetResultStatus::NotFound,
            maybe_serialized_entry: None,
        }
    }
}

impl From<GetResultStatus> for JsonString {
    fn from(get_result_status: GetResultStatus) -> JsonString {
        JsonString::from(
            serde_json::to_string(&get_result_status).expect("could not Jsonify GetResultStatus"),
        )
    }
}

impl From<JsonString> for GetResultStatus {
    fn from(json_string: JsonString) -> GetResultStatus {
        serde_json::from_str(&String::from(json_string))
            .expect("could not deserialize GetStatusResult")
    }
}

impl From<GetEntryResult> for JsonString {
    fn from(v: GetEntryResult) -> JsonString {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for GetEntryResult {
    type Error = HolochainError;
    fn try_from(json_string: JsonString) -> HcResult<GetEntryResult> {
        default_try_from_json(json_string)
    }
}
