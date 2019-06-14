use super::Dispatch;
use error::ZomeApiResult;
use holochain_core_types::entry::cap_entries::{CapFunctions, CapabilityType};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_utils::api_serialization::capabilities::{
    CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
};

/// Adds a capability grant to the local chain
pub fn commit_capability_grant<S: Into<String>>(
    id: S,
    cap_type: CapabilityType,
    assignees: Option<Vec<Address>>,
    functions: CapFunctions,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityGrant.with_input(CommitCapabilityGrantArgs {
        id: id.into(),
        cap_type,
        assignees,
        functions,
    })
}

/// Adds a capability claim to the local chain
pub fn commit_capability_claim<S: Into<String>>(
    id: S,
    grantor: Address,
    token: Address,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityClaim.with_input(CommitCapabilityClaimArgs {
        id: id.into(),
        grantor,
        token,
    })
}
