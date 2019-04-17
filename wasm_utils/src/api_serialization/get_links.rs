use holochain_core_types::{
    cas::content::Address, chain_header::ChainHeader, error::HolochainError, json::*, time::Timeout,
};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
    pub options: GetLinksOptions,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq, Eq, Hash)]
pub enum LinksStatusRequestKind {
    Live,
    Deleted,
    All,
}
impl Default for LinksStatusRequestKind {
    fn default() -> Self {
        LinksStatusRequestKind::Live
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq, Hash, Eq)]
pub struct GetLinksOptions {
    pub status_request: LinksStatusRequestKind,
    pub headers: bool,
    pub timeout: Timeout,
}
impl Default for GetLinksOptions {
    fn default() -> Self {
        GetLinksOptions {
            status_request: LinksStatusRequestKind::default(),
            headers: false,
            timeout: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct LinksResult {
    pub address: Address,
    pub headers: Vec<ChainHeader>,
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct GetLinksResult {
    links: Vec<LinksResult>,
}

impl GetLinksResult {
    pub fn new(links: Vec<LinksResult>) -> GetLinksResult {
        GetLinksResult { links }
    }

    pub fn addresses(&self) -> Vec<Address> {
        self.links.iter().map(|s| s.address.clone()).collect()
    }
}
