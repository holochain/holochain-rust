use dht::actions::add_link::*;
use futures::executor::block_on;
use holochain_core_types::error::HolochainError;
use holochain_core_types::error::ZomeApiInternalResult;
use holochain_core_types::json::JsonString;
use holochain_wasm_utils::api_serialization::link_entries::{LinkEntriesArgs};
use nucleus::ribosome::Runtime;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};
use std::convert::TryFrom;

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: LinkEntriesArgs
/// Returns a serialized LinkEntriesResult
pub fn invoke_link_entries(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };

    // Wait for future to be resolved
    let task_result: Result<(), HolochainError> =
        block_on(add_link(&input.to_link(), &runtime.context));

    let result = match task_result {
        Ok(_) => ZomeApiInternalResult::success(JsonString::null()),
        Err(e) => ZomeApiInternalResult::failure(&e.to_string()),
    };

    runtime.store_as_json_string(result)
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use agent::actions::commit::commit_entry;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent, entry::test_entry,
    };
    use holochain_wasm_utils::api_serialization::link_entries::*;
    use instance::tests::{test_context_and_logger, test_instance};
    use nucleus::ribosome::{
        api::{tests::*, ZomeApiFunction},
        Defn,
    };
    use holochain_core_types::json::JsonString;
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_link_args_bytes() -> Vec<u8> {
        let entry = test_entry();

        let args = LinkEntriesArgs {
            base: entry.address(),
            target: entry.address(),
            tag: String::from("test-tag"),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        JsonString::from(test_entry().serialize()).into_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    fn errors_if_base_is_not_present() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::LinkEntries.as_str(),
            test_link_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from(r#"{"ok":false,"error":"ErrorGeneric(\"Base for link not found\")"}"#.to_string()
                + "\u{0}"),
        );
    }

    #[test]
    fn returns_ok_if_base_is_present() {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::LinkEntries.as_str());
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability(),
            wasm.clone(),
        );

        let dna_name = &dna.name.to_string().clone();
        let instance = test_instance(dna).expect("Could not create test instance");

        let (context, _) = test_context_and_logger("joan");
        let initialized_context = instance.initialize_context(context);

        block_on(commit_entry(
            test_entry(),
            &initialized_context.action_channel.clone(),
            &initialized_context,
        )).expect("Could not commit entry for testing");

        let call_result = test_zome_api_function_call(
            &dna_name,
            initialized_context,
            &instance,
            &wasm,
            test_link_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from(r#"{"ok":true,"error":""}"#.to_string() + "\u{0}"),
        );
    }

}
