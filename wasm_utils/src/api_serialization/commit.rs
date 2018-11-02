use holochain_core_types::cas::content::Address;

/// Struct for input data received when Commit API function is invoked
#[derive(Deserialize, Default, Debug, Serialize)]
pub struct CommitEntryArgs {
    pub entry_type_name: String,
    pub entry_value: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct CommitEntryResult {
    pub address: Address,
}
impl CommitEntryResult {
    pub fn new(address: Address) -> Self {
        CommitEntryResult { address }
    }
}
