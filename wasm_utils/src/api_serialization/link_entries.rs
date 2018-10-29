use holochain_core_types::{cas::content::Address, links_entry::Link};
use holochain_core_types::json::*;
use holochain_core_types::error::HolochainError;
use std::convert::TryFrom;

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct LinkEntriesArgs {
    pub base: Address,
    pub target: Address,
    pub tag: String,
}

impl From<LinkEntriesArgs> for JsonString {
    fn from(v: LinkEntriesArgs) -> Self {
        default_to_json(v)
    }
}

impl TryFrom<JsonString> for LinkEntriesArgs {
    type Error = HolochainError;
    fn try_from(j: JsonString) -> Result<Self, Self::Error> {
        default_try_from_json(j)
    }
}

impl LinkEntriesArgs {
    pub fn to_link(&self) -> Link {
        Link::new(&self.base, &self.target, &self.tag)
    }
}
