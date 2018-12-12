use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    crud_status::CrudStatus,
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
    json::*,
};
use std::collections::HashMap;

#[derive(Deserialize, Debug, Serialize, DefaultJson, Clone, PartialEq)]
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
        GetEntryOptions { status_request }
    }
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct GetEntryArgs {
    pub address: Address,
    pub options: GetEntryOptions,
}

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct EntryHistory {
    pub addresses: Vec<Address>,
    pub entries: Vec<Entry>,
    pub crud_status: Vec<CrudStatus>,
    pub crud_links: HashMap<Address, Address>,
}

impl EntryHistory {
    pub fn new() -> Self {
        EntryHistory {
            addresses: Vec::new(),
            entries: Vec::new(),
            crud_status: Vec::new(),
            crud_links: HashMap::new(),
        }
    }

    pub fn push(&mut self, entry_with_meta: &EntryWithMeta) {
        let address = entry_with_meta.entry.address();
        self.addresses.push(address.clone());
        self.entries.push(entry_with_meta.entry.clone());
        self.crud_status.push(entry_with_meta.crud_status);
        if let Some(new_address) = entry_with_meta.maybe_crud_link.clone() {
            self.crud_links.insert(address, new_address);
        }
    }
}
