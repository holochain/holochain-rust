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
    pub validation_failure: String,
}

impl CommitOutputStruct {
    pub fn success(address: Address) -> CommitOutputStruct {
        CommitOutputStruct {
            address,
            validation_failure: String::from(""),
        }
    }

    pub fn failure(validation_failure: String) -> CommitOutputStruct {
        CommitOutputStruct {
            address: HashString::from(""),
            validation_failure,
        }
    }
}