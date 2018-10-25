#[derive(Deserialize, Default, Debug, Serialize)]
pub struct HashEntryArgs {
    pub entry_type_name: String,
    pub entry_value: String,
}
