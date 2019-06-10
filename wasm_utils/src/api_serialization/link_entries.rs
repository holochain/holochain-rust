use holochain_core_types::link::Link;

use lib3h_persistence_api::{cas::content::Address, error::PersistenceError, json::*};

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
