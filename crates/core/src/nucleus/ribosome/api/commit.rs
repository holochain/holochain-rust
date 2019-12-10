use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::author_entry::author_entry,
};
use holochain_core_types::error::HolochainError;

use holochain_wasm_utils::api_serialization::commit_entry::{CommitEntryArgs, CommitEntryResult};

use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::CommitAppEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: CommitEntryArg
/// Returns an HcApiReturnCode as I64
pub fn invoke_commit_app_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let commit_entry_arg = match CommitEntryArgs::try_from(args_str.clone()) {
        Ok(commit_entry_arg_input) => commit_entry_arg_input,
        // Exit on error
        Err(error) => {
            log_error!(
                context,
                "zome: invoke_commit_app_commit_entry_arg failed to \
                 deserialize Entry: {:?} with error {:?}",
                args_str,
                error
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    // Wait for future to be resolved
    let task_result: Result<CommitEntryResult, HolochainError> = context.block_on(author_entry(
        &commit_entry_arg.entry(),
        None,
        &context,
        &commit_entry_arg.options().provenance(),
    ));

    runtime.store_result(task_result)
}

#[cfg(test)]
pub mod tests {
    use crate::nucleus::ribosome::{
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
