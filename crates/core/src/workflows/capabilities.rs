use crate::{agent::actions::commit::commit_entry, context::Context, NEW_RELIC_LICENSE_KEY};
use holochain_core_types::{
    entry::{
        cap_entries::{CapTokenClaim, CapTokenGrant},
        Entry,
    },
    error::HolochainError,
};
use holochain_persistence_api::cas::content::Address;
use std::sync::Arc;

use holochain_wasm_types::capabilities::{
    CommitCapabilityClaimArgs, CommitCapabilityGrantArgs,
};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_commit_capability_grant(
    context: Arc<Context>,
    args: CommitCapabilityGrantArgs,
) -> Result<Address, HolochainError> {
    match CapTokenGrant::create(&args.id, args.cap_type, args.assignees, args.functions) {
        Ok(grant) => context.block_on(commit_entry(Entry::CapTokenGrant(grant), None, &context)),
        Err(err) => Err(HolochainError::ErrorGeneric(format!(
            "Unable to commit capability grant: {}",
            err
        ))),
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_commit_capability_claim(
    context: Arc<Context>,
    args: CommitCapabilityClaimArgs,
) -> Result<Address, HolochainError> {
    let claim = CapTokenClaim::new(args.id, args.grantor, args.token);
    context.block_on(commit_entry(Entry::CapTokenClaim(claim), None, &context))
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
