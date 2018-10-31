#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryArgs {
    pub entry_type_name: String,
    pub limit: u32,
}
