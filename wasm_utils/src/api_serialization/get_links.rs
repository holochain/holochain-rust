use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
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
