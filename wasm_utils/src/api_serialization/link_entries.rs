use holochain_core_types::{cas::content::Address, error::HolochainError, json::*, link::Link};

#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct LinkEntriesArgs {
    pub base: Address,
    pub target: Address,
    pub link_type: String,
}

impl LinkEntriesArgs {
    pub fn to_link(&self) -> Link {
        Link::new(&self.base, &self.target, &self.link_type)
    }
}
