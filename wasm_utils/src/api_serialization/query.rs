use holochain_core_types::{error::HolochainError, hash::HashString, json::*};
use std::convert::TryFrom;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct QueryArgs {
    pub entry_type_name: String,
    pub limit: u32,
}

impl From<QueryArgs> for JsonString {
    fn from(v: QueryArgs) -> JsonString {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for QueryArgs {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

pub type QueryResult = Vec<HashString>;
