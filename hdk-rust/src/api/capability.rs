use holochain_core_types::{
    entry::cap_entries::{CapabilityType, CapFunctions},
    cas::content::Address,
};
use holochain_wasm_utils::api_serialization::capabilities::{CommitCapabilityGrantArgs, CommitCapabilityClaimArgs};
use error::ZomeApiResult;
use super::Dispatch;

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
