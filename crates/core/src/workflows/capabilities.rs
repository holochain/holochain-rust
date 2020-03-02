use crate::{agent::actions::commit::commit_entry};
use holochain_core_types::{
    entry::{
        cap_entries::{CapTokenClaim, CapTokenGrant},
        Entry,
    },
    error::HolochainError,
};
use holochain_persistence_api::cas::content::Address;
use holochain_wasm_types::capabilities::{
    CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
};
use std::sync::Arc;
use crate::context::Context;
use crate::workflows::WorkflowResult;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn commit_capability_grant_workflow(
    context: Arc<Context>,
    args: &CommitCapabilityGrantArgs,
) -> WorkflowResult<Address> {
    match CapTokenGrant::create(&args.id, args.cap_type.clone(), args.assignees.clone(), args.functions.clone()) {
        Ok(grant) => commit_entry(Entry::CapTokenGrant(grant), None, &context).await,
        Err(err) => Err(HolochainError::ErrorGeneric(format!(
            "Unable to commit capability grant: {}",
            err
        ))),
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn commit_capability_claim_workflow(
    context: Arc<Context>,
    args: &CommitCapabilityClaimArgs,
) -> WorkflowResult<Address> {
    let claim = CapTokenClaim::new(args.id.clone(), args.grantor.clone(), args.token.clone());
    commit_entry(Entry::CapTokenClaim(claim), None, &context).await
}

#[cfg(test)]
pub mod tests {
    use crate::wasm_engine::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{entry::cap_entries::CapabilityType, error::ZomeApiInternalResult};
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::Address;
    use holochain_wasm_types::capabilities::{
        CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
    };
    use std::collections::BTreeMap;

    /// dummy args
    pub fn test_commit_capability_grant_args_bytes() -> Vec<u8> {
        let mut functions = BTreeMap::new();
        functions.insert("test_zome".to_string(), vec!["test_function".to_string()]);
        let grant_args = CommitCapabilityGrantArgs {
            id: "some_id".to_string(),
            cap_type: CapabilityType::Assigned,
            assignees: Some(vec![Address::from("fake address")]),
            functions,
        };

        JsonString::from(grant_args).to_bytes()
    }

    pub fn test_commit_capability_claim_args_bytes() -> Vec<u8> {
        let claim_args = CommitCapabilityClaimArgs {
            id: "some_id".to_string(),
            grantor: Address::from("fake grantor"),
            token: Address::from("fake"),
        };

        JsonString::from(claim_args).to_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit_capability_grant action and get the result from WASM
    fn test_commit_capability_grant_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::CommitCapabilityGrant.as_str(),
            test_commit_capability_grant_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(JsonString::from(ZomeApiInternalResult::success(
                    Address::from("Qma8KWBHZwiXNBJ4PBtT4uDUVgPAyUJASHumThZMTPAAJe")
                ))) + "\u{0}")
            ),
        );
    }

    #[test]
    /// test that we can round trip bytes through a commit_capability_claim action and get the result from WASM
    fn test_commit_capability_claim_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::CommitCapabilityClaim.as_str(),
            test_commit_capability_claim_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(JsonString::from(ZomeApiInternalResult::success(
                    Address::from("QmeuneB3iJjcGMkei7N8kyoc7Ubi4ab3xMNPYXSse2vdm5")
                ))) + "\u{0}")
            ),
        );
    }
}
