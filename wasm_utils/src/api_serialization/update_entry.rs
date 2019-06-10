use holochain_core_types::entry::Entry;
use lib3h_persistence_api::{cas::content::Address, error::PersistenceError, json::*};

/// Struct for input data received when Zome API function update_entry() is invoked
#[derive(Deserialize, Clone, PartialEq, Debug, Serialize, DefaultJson)]
pub struct UpdateEntryArgs {
    pub new_entry: Entry,
    pub address: Address,
}
