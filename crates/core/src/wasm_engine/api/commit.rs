use crate::{
    workflows::author_entry::author_entry,
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::commit_entry::{CommitEntryArgs, CommitEntryResult};
use std::sync::Arc;
use crate::context::Context;

/// ZomeApiFunction::CommitAppEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: CommitEntryArg
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_commit_app_entry(
    context: Arc<Context>,
    commit_entry_arg: CommitEntryArgs,
) -> Result<CommitEntryResult, HolochainError> {
    let span = context
        .tracer
        .span("hdk invoke_commit_app_entry")
        .tag(ht::Tag::new(
            "CommitEntryArgs",
            format!("{:?}", commit_entry_arg),
        ))
        .start()
        .into();
    let _spanguard = ht::push_span(span);

    // Wait for future to be resolved
    context.block_on(author_entry(
        &commit_entry_arg.entry(),
        None,
        &context,
        &commit_entry_arg.options().provenance(),
    ))
}

#[cfg(test)]
pub mod tests {
    use crate::wasm_engine::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{
        entry::test_entry,
        error::ZomeApiInternalResult,
        signature::{Provenance, Signature},
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_wasm_utils::api_serialization::commit_entry::{
        CommitEntryArgs, CommitEntryOptions, CommitEntryResult,
    };

    /// dummy commit with provenance args from standard test entry
    pub fn test_commit_entry_args_bytes() -> Vec<u8> {
        let entry = test_entry();
        let address: Address = entry.address();

        let agent_nick = "counter-signer";
        let agent_id = test_utils::mock_signing::registered_test_agent(agent_nick);

        let signature = Signature::from(test_utils::mock_signing::mock_signer(
            String::from(address.clone()),
            &agent_id,
        ));

        let provenances = vec![Provenance::new(agent_id.address(), signature)];
        let serialized_commit_entry_arg =
            CommitEntryArgs::new(entry, CommitEntryOptions::new(provenances));
        JsonString::from(serialized_commit_entry_arg).to_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit action with
    /// additional provenance and get the result from WASM
    fn test_commit_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::CommitAppEntry.as_str(),
            test_commit_entry_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(JsonString::from(ZomeApiInternalResult::success(
                    CommitEntryResult::new(Address::from(
                        "Qma6RfzvZRL127UCEVEktPhQ7YSS1inxEFw7SjEsfMJcrq"
                    ))
                ))) + "\u{0}")
            ),
        );
    }
}
