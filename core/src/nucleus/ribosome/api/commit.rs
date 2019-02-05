use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::author_entry::author_entry,
};
use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::CommitAppEntry function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: CommitArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_commit_app_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let entry = match Entry::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.clone().log(format!(
                "err/zome: invoke_commit_app_entry failed to deserialize Entry: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    // Wait for future to be resolved
    let task_result: Result<Address, HolochainError> = zome_call_data
        .context
        .clone()
        .block_on(author_entry(&entry, None, &zome_call_data.context));

    runtime.store_result(task_result)
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use crate::nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::{
        cas::content::Address,
        entry::{test_entry, Entry},
        error::ZomeApiInternalResult,
        json::JsonString,
    };

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        let entry = test_entry();

        let serialized_entry = Entry::from(entry);
        JsonString::from(serialized_entry).into_bytes()
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
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(
                    Address::from("Qma6RfzvZRL127UCEVEktPhQ7YSS1inxEFw7SjEsfMJcrq")
                ))) + "\u{0}"
            ),
        );
    }
}
