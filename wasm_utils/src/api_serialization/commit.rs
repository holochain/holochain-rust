use holochain_core_types::{
    cas::content::Address, entry::SerializedEntry, hash::HashString, json::JsonString,
};
use serde_json;

pub type CommitEntryArgs = SerializedEntry;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct CommitEntryResult {
    pub address: Address,
    pub validation_failure: String,
}

impl CommitEntryResult {
    pub fn success(address: Address) -> CommitEntryResult {
        CommitEntryResult {
            address,
            validation_failure: String::from(""),
        }
    }

    pub fn failure(validation_failure: String) -> CommitEntryResult {
        CommitEntryResult {
            address: HashString::from(""),
            validation_failure,
        }
    }
}

impl From<CommitEntryResult> for JsonString {
    fn from(commit_entry_result: CommitEntryResult) -> JsonString {
        JsonString::from(
            serde_json::to_string(&commit_entry_result)
                .expect("could not Jsonify CommitEntryResult"),
        )
    }
}
