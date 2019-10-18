use holochain_core_types::link::Link;
use holochain_json_api::{error::JsonError, json::*};
use holochain_persistence_api::cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct LinkEntriesArgs {
    pub base: Address,
    pub target: Address,
    pub link_type: String,
    pub tag: String,
}

impl LinkEntriesArgs {
    pub fn to_link(&self) -> Link {
        Link::new(&self.base, &self.target, &self.link_type, &self.tag)
    }
}
