use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct QueryArgs {
    pub entry_type_name: String,
    pub limit: u32,
}

pub type QueryResult = Vec<Address>;
