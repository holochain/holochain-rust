use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

#[derive(Deserialize, Default, Debug, Serialize, Clone, PartialEq, Eq, Hash, DefaultJson)]
pub struct GetLinksArgs {
    pub entry_address: Address,
    pub tag: String,
}
