extern crate futures;
use agent::{
    actions::commit::*,
    state::AgentState,
};
use futures::{executor::block_on, FutureExt};
use holochain_core_types::{
    cas::content::Address,
    entry::Entry, entry_type::EntryType,
    error::HolochainError,
    hash::HashString
};
use holochain_wasm_utils::api_serialization::{
    commit::{CommitEntryArgs, CommitOutputStruct},
    validation::{EntryAction, EntryLifecycle, ValidationData}
};
use nucleus::{actions::validate::*, ribosome::api::Runtime};
use serde_json;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

fn build_validation_data_commit(
    _entry: Entry,
    _entry_type: EntryType,
    _state: &AgentState,
) -> ValidationData {
    //
    // TODO: populate validation data with with chain content
    // I have left this out because filling the valiation data with
    // chain headers and entries does not work as long as ValidationData
    // is defined with the type copies i've put in wasm_utils/src/api_serialization/validation.rs.
    // Doing this right requires a refactoring in which I extract all these types
    // into a separate create ("core_types") that can be used from holochain core
    // and the HDK.

    //let agent_key = state.keys().expect("Can't commit entry without agent key");
    ValidationData {
        chain_header: None, //Some(new_header),
        sources: vec![HashString::from("<insert your agent key here>")],
        source_chain_entries: None,
        source_chain_headers: None,
        custom: None,
        lifecycle: EntryLifecycle::Chain,
        action: EntryAction::Commit,
    }
}

/// ZomeApiFunction::CommitAppEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: CommitArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_commit_app_entry(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
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
    let validation_data = build_validation_data_commit(
        entry.clone(),
        entry_type.clone(),
        &runtime.context.state().unwrap().agent(),
    );

    // Wait for future to be resolved
    let task_result: Result<Address, HolochainError> = block_on(
        // First validate entry:
        validate_entry(
            entry_type.clone(),
            entry.clone(),
            validation_data,
            &runtime.context)
            // if successful, commit entry:
            .and_then(|_| commit_entry(entry.clone(), &runtime.context.action_channel, &runtime.context)),
    );

    let maybe_json = match task_result {
        Ok(address) => serde_json::to_string(&CommitOutputStruct::success(address)),
        Err(HolochainError::ValidationFailed(fail_string)) => serde_json::to_string(&CommitOutputStruct::failure(fail_string)),
        Err(error_string) => {
            let error_report = ribosome_error_report!(format!(
                "Call to `hc_commit_entry()` failed: {}",
                error_string
            ));

            serde_json::to_string(&error_report.to_string())
            // TODO #394 - In release return error_string directly and not a RibosomeErrorReport
            // Ok(error_string)
        }
    };
    
    match maybe_json {
        Ok(json) => runtime.store_utf8(&json),
        Err(_) => ribosome_error_code!(ResponseSerializationFailed),
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
        api::{commit::CommitEntryArgs, tests::test_zome_api_function_runtime, ZomeApiFunction},
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
        let (runtime, _) = test_zome_api_function_runtime(
            ZomeApiFunction::CommitAppEntry.as_str(),
            test_commit_args_bytes(),
        );

        assert_eq!(
            runtime.result,
            format!(r#"{{"address":"{}","validation_failure":""}}"#, test_entry().address()) + "\u{0}",
        );
    }

}
