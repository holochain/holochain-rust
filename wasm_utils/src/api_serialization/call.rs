use holochain_core_types::{dna::capabilities::CapabilityCall, error::HolochainError, json::*};

pub const THIS_INSTANCE: &str = "__hdk_this_instance";

/// Struct for input data received when Zome API function call() is invoked
#[derive(Deserialize, Default, Clone, PartialEq, Eq, Hash, Debug, Serialize, DefaultJson)]
pub struct ZomeFnCallArgs {
    pub instance_handle: String,
    pub zome_name: String,
    pub cap: Option<CapabilityCall>,
    pub fn_name: String,
    pub fn_args: String,
}
