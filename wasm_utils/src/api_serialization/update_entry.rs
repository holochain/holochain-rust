use holochain_core_types::{
    error::HolochainError, json::*,
    cas::content::Address,
    entry::SerializedEntry,

};

/// Struct for input data received when Zome API function update_entry() is invoked
#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct UpdateEntryArgs {
    pub new_entry: SerializedEntry,
    pub address: Address,
}
