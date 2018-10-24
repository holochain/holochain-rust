use holochain_core_types::{cas::content::Address, entry::SerializedEntry, json::JsonString};
use serde_json;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct GetEntryArgs {
    pub address: Address,
}

impl From<GetEntryArgs> for JsonString {
    fn from(get_entry_args: GetEntryArgs) -> JsonString {
        JsonString::from(
            serde_json::to_string(&get_entry_args).expect("could not Jsonify GetEntryArgs"),
        )
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResultStatus {
    Found,
    NotFound,
}

#[derive(Debug, Serialize, Deserialize)]
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
    fn from(get_entry_result: GetEntryResult) -> JsonString {
        JsonString::from(
            serde_json::to_string(&get_entry_result).expect("could not Jsonify GetEntryResult"),
        )
    }
}
