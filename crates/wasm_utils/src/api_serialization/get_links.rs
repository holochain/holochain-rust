use holochain_core_types::{chain_header::ChainHeader, crud_status::CrudStatus, time::Timeout};
use holochain_json_api::{error::JsonError, json::*};
use holochain_persistence_api::cas::content::Address;

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub link_type: String,
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

#[derive(Deserialize, Clone, Serialize, Debug, DefaultJson, PartialEq)]
pub struct LinksResult {
    pub address: Address,
    pub headers: Vec<ChainHeader>,
    pub tag: String,
    pub status: CrudStatus,
}

#[derive(Deserialize, Clone, Serialize, Debug, DefaultJson)]
pub struct GetLinksResult {
    links: Vec<LinksResult>,
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct GetLinksResultCount {
    pub count: usize,
}

impl GetLinksResult {
    pub fn new(links: Vec<LinksResult>) -> GetLinksResult {
        GetLinksResult { links }
    }

    pub fn tags(&self) -> Vec<String> {
        self.links.iter().map(|s| s.tag.clone()).collect()
    }

    pub fn links(&self) -> Vec<LinksResult> {
        self.links.clone()
    }

    pub fn addresses(&self) -> Vec<Address> {
        self.links.iter().map(|s| s.address.clone()).collect()
    }
}
