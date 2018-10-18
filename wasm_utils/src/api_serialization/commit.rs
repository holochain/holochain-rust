use holochain_core_types::{cas::content::Address, hash::HashString};
use holochain_core_types::json::JsonString;
use serde_json;

/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug, Serialize)]
pub struct CommitEntryArgs {
    pub entry_type_name: String,
    pub entry_value: String,
}
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
        JsonString::from(serde_json::to_string(&commit_entry_result).expect("could not Jsonify CommitEntryResult"))
    }
}
