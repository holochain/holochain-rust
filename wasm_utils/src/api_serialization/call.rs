use holochain_core_types::{cas::content::Address, error::HolochainError, json::*};

/// Struct for input data received when Zome API function call() is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct ZomeFnCallArgs {
    pub zome_name: String,
    pub cap_name: String, // temporary till we move fn declarations out of capabilities
    pub cap_token: Address,
    pub fn_name: String,
    pub fn_args: String,
}
