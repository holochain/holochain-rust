use holochain_core_types::{
    cas::content::Address,
    entry::cap_entries::{CapFunctions, CapabilityType},
    error::HolochainError,
    json::*,
};

// arguments required for calling commit_capability_grant
#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct CommitCapabilityGrantArgs {
    pub id: String,
    pub cap_type: CapabilityType,
    pub assignees: Option<Vec<Address>>,
    pub functions: CapFunctions,
}

// arguments required for calling commit_capability_claim
#[derive(Deserialize, Default, Debug, Serialize, DefaultJson)]
pub struct CommitCapabilityClaimArgs {
    pub id: String,
    pub grantor: Address,
    pub token: Address,
}
