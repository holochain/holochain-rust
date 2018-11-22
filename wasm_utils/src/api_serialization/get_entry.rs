
use holochain_core_types::{
    crud_status::CrudStatus,
    entry::SerializedEntry,
    cas::content::Address,
    error::HolochainError,
    json::*,
};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub enum StatusRequestKind {
    Initial,
    Latest,
    All,
}
impl Default for StatusRequestKind {
    fn default() -> Self {
        StatusRequestKind::Latest
    }
}


#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone)]
pub struct GetEntryOptions {
    pub status_request: StatusRequestKind,
}

impl Default for GetEntryOptions {
    fn default() -> Self {
        GetEntryOptions {
            status_request: StatusRequestKind::default(),
        }
    }
}

impl GetEntryOptions {
    pub fn new(status_request: StatusRequestKind) -> Self {
        GetEntryOptions {
            status_request,
        }
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct GetEntryArgs {
    pub address: Address,
    pub options: GetEntryOptions,
}


#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct GetEntryResult {
    pub addresses: Vec<Address>,
    pub entries: Vec<SerializedEntry>,
    pub crud_status: Vec<CrudStatus>,
    pub crud_links: HashMap<Address, Address>,
}

impl GetEntryResult {
    pub fn new() -> Self {
        GetEntryResult {
            addresses: Vec::new(),
            entries: Vec::new(),
            crud_status: Vec::new(),
            crud_links: HashMap::new(),
        }
    }
}