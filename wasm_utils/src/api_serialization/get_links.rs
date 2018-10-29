use holochain_core_types::cas::content::Address;
use holochain_core_types::json::*;
use std::convert::TryFrom;
use holochain_core_types::error::HolochainError;

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

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct GetLinksResult {
    pub ok: bool,
    pub links: Vec<Address>,
    pub error: String,
}

impl TryFrom<JsonString> for GetLinksResult {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl From<GetLinksResult> for JsonString {
    fn from(v: GetLinksResult) -> Self {
        default_to_json(v)
    }
}
