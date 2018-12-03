use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError, json::*};

/// Struct for input data received when Zome API function update_entry() is invoked
#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct UpdateEntryArgs {
    pub new_entry: Entry,
    pub address: Address,
}
