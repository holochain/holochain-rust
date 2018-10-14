use holochain_core_types::{
    cas::content::Address,
    hash::HashString,
};

/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug, Serialize)]
pub struct CommitEntryArgs {
    pub entry_type_name: String,
    pub entry_value: String,
}
#[derive(Deserialize, Serialize, Default,Debug)]
pub struct CommitOutputStruct {
    pub address: Address,
    pub error: String,
}

impl CommitOutputStruct {
    pub fn new() -> CommitOutputStruct {
        CommitOutputStruct {
            address: HashString::from(""),
            error: String::from(""),
        }
    }
}