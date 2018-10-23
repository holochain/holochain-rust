use holochain_core_types::{
    cas::content::Address,
    links_entry::Link,
};

#[derive(Deserialize, Default, Debug, Serialize)]
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

#[derive(Deserialize, Default, Debug, Serialize)]
pub struct LinkEntriesResult {
    pub ok: bool,
    pub error: String,
}
