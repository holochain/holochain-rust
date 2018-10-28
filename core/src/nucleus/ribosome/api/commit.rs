use agent::actions::commit::*;
use futures::{executor::block_on, FutureExt};
use holochain_core_types::{
    cas::content::Address,
    entry::Entry,
    entry_type::EntryType,
    error::HolochainError,
    hash::HashString,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use holochain_wasm_utils::api_serialization::commit::{CommitEntryArgs, CommitEntryResult};
use nucleus::{
    actions::{build_validation_package::*, validate::*},
    ribosome::{Runtime, api::ZomeApiResult},
};
use serde_json;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::CommitAppEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: CommitArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_commit_app_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: CommitEntryArgs = match serde_json::from_str(&args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    // Create Chain Entry
    let entry_type =
        EntryType::from_str(&input.entry_type_name).expect("could not create EntryType from str");
    let entry = Entry::new(&entry_type, &input.entry_value);

    // Wait for future to be resolved
    let task_result: Result<Address, HolochainError> = block_on(
        // 1. Build the context needed for validation of the entry
        build_validation_package(&entry, &runtime.context)
            .and_then(|validation_package| {
                Ok(ValidationData {
                    package: validation_package,
                    sources: vec![HashString::from("<insert your agent key here>")],
                    lifecycle: EntryLifecycle::Chain,
                    action: EntryAction::Commit,
                })
            })
            // 2. Validate the entry
            .and_then(|validation_data| {
                validate_entry(
                    entry_type.clone(),
                    entry.clone(),
                    validation_data,
                    &runtime.context)
            })
            // 3. Commit the valid entry to chain and DHT
            .and_then(|_| commit_entry(entry.clone(), &runtime.context.action_channel, &runtime.context)),
    );

    match task_result {
        Err(hc_err) => runtime.store_as_json(core_error!(hc_err)),
        Ok(address) => runtime.store_as_json(CommitEntryResult::success(address)),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use holochain_core_types::{
        cas::content::AddressableContent, entry::test_entry, entry_type::test_entry_type,
    };
    use nucleus::ribosome::{
        api::{commit::CommitEntryArgs, tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use serde_json;

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        let entry_type = test_entry_type();
        let entry = test_entry();

        let args = CommitEntryArgs {
            entry_type_name: entry_type.to_string(),
            entry_value: entry.value().to_owned(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    fn test_commit_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::CommitAppEntry.as_str(),
            test_commit_args_bytes(),
        );

        assert_eq!(
            call_result,
            format!(
                r#"{{"address":"{}","validation_failure":""}}"#,
                test_entry().address()
            ) + "\u{0}",
        );
    }

}
