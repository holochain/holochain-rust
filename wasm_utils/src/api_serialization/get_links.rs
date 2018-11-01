use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};
use std::convert::TryFrom;

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
}

impl From<GetLinksArgs> for JsonString {
    fn from(v: GetLinksArgs) -> Self {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for GetLinksArgs {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}
