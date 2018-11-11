use holochain_core_types::{
    cas::content::Address, entry::link_add::Link, error::HolochainError, json::*,
};

#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct LinkEntriesArgs {
    pub base: Address,
    pub target: Address,
    pub tag: String,
}

impl LinkEntriesArgs {
    pub fn to_link(&self) -> Link {
        Link::new(&self.base, &self.target, &self.tag)
    }
}
