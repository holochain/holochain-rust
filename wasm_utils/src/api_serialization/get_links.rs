use crate::api_serialization::get_entry::GetEntryOptions;
use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
}

// options for get links is the same as get entry for now. May change later
pub type GetLinksOptions = GetEntryOptions;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetLinksLoadElement<T> {
    pub address: Address,
    pub entry: T,
}

pub type GetLinksLoadResult<T> = Vec<GetLinksLoadElement<T>>;
