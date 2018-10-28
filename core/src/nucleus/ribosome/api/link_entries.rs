use dht::actions::add_link::*;
use futures::executor::block_on;
use holochain_core_types::error::HolochainError;
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use nucleus::ribosome::{Runtime, api::ZomeApiResult};
use serde_json;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: LinkEntriesArgs
/// Returns 'Success' return code on success.
pub fn invoke_link_entries(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: LinkEntriesArgs = match serde_json::from_str(&args_str) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };
    // Wait for add_link() future to be resolved
    let task_result: Result<(), HolochainError> =
        block_on(add_link(&input.to_link(), &runtime.context));
    // Write result in wasm memory
    match task_result {
        Err(hc_err) => runtime.store_as_json(core_error!(hc_err)),
        Ok(()) => ribosome_success!(),
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use agent::actions::commit::commit_entry;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent, entry::test_entry, entry_type::test_entry_type,
        error::CoreError,
    };
    use holochain_wasm_utils::api_serialization::{commit::CommitEntryArgs, link_entries::*};
    use instance::tests::{test_context_and_logger, test_instance};
    use nucleus::ribosome::{
        api::{tests::*, ZomeApiFunction},
        Defn,
    };
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
    fn errors_if_base_is_not_present() {
        let (mut call_result, _) = test_zome_api_function(
            ZomeApiFunction::LinkEntries.as_str(),
            test_link_args_bytes(),
        );
        call_result.pop(); // Remove trailing character
        let core_err: CoreError = serde_json::from_str(&call_result).expect("valid CoreError json str");
        assert_eq!(
            "ErrorGeneric(\"Base for link not found\")",
            core_err.kind.to_string(),
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
            r#"{"ok":true,"error":""}"#.to_string() + "\u{0}",
        );
    }

}
