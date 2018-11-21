use holochain_core_types::{cas::content::Address, crud_status::CrudStatus, entry::Entry};
use std::iter::Map;

// empty for now, need to implement get options
#[derive(Deserialize, Debug, Serialize)]
pub struct GetEntryOptions {}

pub struct GetEntryResult {
    addresses: Vec<Address>,
    entries: Vec<Entry>,
    crud_statuses: Vec<CrudStatus>,
    crud_links: Map<Address, Address>,
}

impl GetEntryResult {
    pub fn new(
        addresses: Vec<Address>,
        entries: Vec<Entry>,
        crud_statuses: Vec<CrudStatus>,
        crud_links: Map<Address, Address>,
    ) -> GetEntryResult {
        GetEntryResult {
            addresses,
            entries,
            crud_statuses,
            crud_links,
        }
    }

    pub fn addresses(&self) -> Vec<Address> {
        self.addresses.clone()
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.entries.clone()
    }

    pub fn crud_statuses(&self) -> Vec<CrudStatus> {
        self.crud_statuses.clone()
    }

    pub fn crud_links(&self) -> Map<Address, Address> {
        self.crud_links.clone()
    }
}
