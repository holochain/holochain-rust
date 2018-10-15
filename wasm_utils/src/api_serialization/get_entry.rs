use holochain_core_types::cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct GetEntryArgs {
    pub address: Address,
}

#[derive(Deserialize, Debug, Serialize)]
pub enum GetResultStatus {
    Found,
    NotFound,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct GetEntryResult {
    pub status: GetResultStatus,
    pub entry: String,
}

impl GetEntryResult {
    pub fn found(entry: String) -> GetEntryResult{
        GetEntryResult {
            status: GetResultStatus::Found,
            entry,
        }
    }

    pub fn not_found() -> GetEntryResult {
        GetEntryResult {
            status: GetResultStatus::NotFound,
            entry: String::from(""),
        }
    }
}