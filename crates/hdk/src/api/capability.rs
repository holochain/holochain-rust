use crate::{error::ZomeApiResult};
use holochain_core_types::entry::cap_entries::{CapFunctions, CapabilityType};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::capabilities::{
    CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
};
use crate::api::hc_commit_capability_grant;
use crate::api::hc_commit_capability_claim;
use holochain_wasmer_guest::host_call;

/// Adds a capability grant to the local chain
pub fn commit_capability_grant<S: Into<String>>(
    id: S,
    cap_type: CapabilityType,
    assignees: Option<Vec<Address>>,
    functions: CapFunctions,
) -> ZomeApiResult<Address> {
    Ok(host_call!(hc_commit_capability_grant, CommitCapabilityGrantArgs {
        id: id.into(),
        cap_type,
        assignees,
        functions,
    })?)
}

/// Adds a capability claim to the local chain
pub fn commit_capability_claim<S: Into<String>>(
    id: S,
    grantor: Address,
    token: Address,
) -> ZomeApiResult<Address> {
    Ok(host_call!(hc_commit_capability_claim, CommitCapabilityClaimArgs {
        id: id.into(),
        grantor,
        token,
    })?)
}
