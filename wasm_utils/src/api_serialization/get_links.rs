use holochain_core_types::{cas::content::Address, error::HolochainError, json::*, time::Timeout};

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
    pub sources: bool,
    pub timeout: Timeout,
}
impl Default for GetLinksOptions {
    fn default() -> Self {
        GetLinksOptions {
            status_request: LinksStatusRequestKind::default(),
            sources: false,
            timeout: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, DefaultJson)]
pub struct GetLinksResult {
    addresses: Vec<Address>,
}

impl GetLinksResult {
    pub fn new(addresses: Vec<Address>) -> GetLinksResult {
        GetLinksResult { addresses }
    }

    pub fn addresses(&self) -> &Vec<Address> {
        &self.addresses
    }
}
