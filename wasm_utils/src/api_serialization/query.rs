use holochain_core_types::cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryArgs {
    pub entry_type_name: String,
    pub limit: u32,
}

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryResult {
    pub addresses: Vec<Address>,
}
