use super::Dispatch;
use error::ZomeApiResult;
use holochain_core_types::{
    cas::content::Address,
    entry::cap_entries::{CapFunctions, CapabilityType},
};
use holochain_wasm_utils::api_serialization::capabilities::{
    CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
};

/// Adds a capability grant to the local chain
pub fn commit_capability_grant<S: Into<String>>(
    id: S,
    cap_type: &CapabilityType,
    assignees: Option<&[Address]>,
    functions: &CapFunctions,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityGrant.with_input(CommitCapabilityGrantArgs {
        id: id.into(),
        cap_type: cap_type.to_owned(),
        assignees: assignees.map(|v| Vec::from(v)),
        functions: functions.to_owned(),
    })
}

/// Adds a capability claim to the local chain
pub fn commit_capability_claim<S: Into<String>>(
    id: S,
    grantor: &Address,
    token: &Address,
) -> ZomeApiResult<Address> {
    Dispatch::CommitCapabilityClaim.with_input(CommitCapabilityClaimArgs {
        id: id.into(),
        grantor: grantor.to_owned(),
        token: token.to_owned(),
    })
}
