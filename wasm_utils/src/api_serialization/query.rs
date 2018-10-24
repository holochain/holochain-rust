use holochain_core_types::hash::HashString;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryArgs {
    pub entry_type_name: String,
    pub limit: u32,
}

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryResult {
    pub hashes: Vec<HashString>,
}
