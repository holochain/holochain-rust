use dht::actions::add_link::*;
use futures::executor::block_on;
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use nucleus::ribosome::{api::ZomeApiResult, Runtime};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: LinkEntriesArgs
pub fn invoke_link_entries(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            println!(
                "invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    // Wait for add_link() future to be resolved
    let result = block_on(add_link(&input.to_link(), &runtime.context));
    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use agent::actions::commit::commit_entry;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent,
        entry::test_entry,
        error::{CoreError, ZomeApiInternalResult},
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::link_entries::*;
    use instance::tests::{test_context_and_logger, test_instance};
    use nucleus::ribosome::{
        api::{tests::*, ZomeApiFunction},
        Defn,
    };
    use serde_json;
    use std::convert::TryFrom;

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
    fn errors_if_base_is_not_present_test() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::LinkEntries.as_str(),
            test_link_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from(
                "{\"ok\":false,\"value\":\"null\",\"error\":\"{\\\"kind\\\":{\\\"ErrorGeneric\\\":\\\"Base for link not found\\\"},\\\"file\\\":\\\"core/src/nucleus/ribosome/runtime.rs\\\",\\\"line\\\":\\\"83\\\"}\"}"
            ),
        );

        let result = ZomeApiInternalResult::try_from(call_result)
            .expect("valid ZomeApiInternalResult JsonString");

        let core_err = CoreError::try_from(result).expect("valid CoreError JsonString");
        assert_eq!("Base for link not found", core_err.kind.to_string(),);
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
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(None))) + "\u{0}"
            ),
        );
    }

}
