use crate::{
    nucleus::ribosome::{api::ZomeApiResult, runtime::Runtime},
    workflows::author_entry::author_entry,
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
};
use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: LinkEntriesArgs
pub fn invoke_link_entries(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let zome_call_data = runtime.zome_call_data()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            zome_call_data.context.log(format!(
                "err/zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let link = input.to_link();
    let link_add = LinkData::from_link(&link, LinkActionKind::ADD);
    let entry = Entry::LinkAdd(link_add);
    // Wait for future to be resolved
    let result: Result<(), HolochainError> = zome_call_data
        .context
        .block_on(author_entry(&entry, None, &zome_call_data.context))
        .map(|_| ());

    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    use test_utils;

    use crate::{
        agent::actions::commit::commit_entry,
        context::Context,
        instance::{tests::test_instance_and_context, Instance},
        nucleus::{
            ribosome::{
                api::{tests::*, ZomeApiFunction},
                Defn,
            },
            tests::*,
        },
    };
    use holochain_core_types::{
        cas::content::AddressableContent,
        entry::{test_entry, Entry},
        error::{CoreError, ZomeApiInternalResult},
        json::JsonString,
    };
    use holochain_wasm_utils::api_serialization::link_entries::*;
    use serde_json;
    use std::{convert::TryFrom, sync::Arc};

    pub fn test_entry_b() -> Entry {
        Entry::App("testEntryTypeB".into(), "test".into())
    }

    /// dummy link_entries args from standard test entry
    pub fn test_link_args_bytes(tag: String) -> Vec<u8> {
        let entry = test_entry();

        let args = LinkEntriesArgs {
            base: entry.address(),
            target: entry.address(),
            tag,
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    pub fn test_link_2_args_bytes(tag: String) -> Vec<u8> {
        let base = test_entry();
        let target = test_entry_b();

        let args = LinkEntriesArgs {
            base: base.address(),
            target: target.address(),
            tag,
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        JsonString::from(test_entry()).into_bytes()
    }

    fn create_test_instance() -> (Instance, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::LinkEntries.as_str());
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability_name(),
            wasm.clone(),
        );

        let netname = Some("create_test_instance");
        test_instance_and_context(dna, netname).expect("Could not create test instance")
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    #[cfg(not(windows))]
    fn errors_if_base_is_not_present_test() {
        // let (call_result, _) = test_zome_api_function(
        //     ZomeApiFunction::LinkEntries.as_str(),
        //     test_link_args_bytes(String::from("test-tag")),
        // );
        //
        // let result = ZomeApiInternalResult::try_from(call_result)
        //     .expect("valid ZomeApiInternalResult JsonString");
        //
        // let core_err = CoreError::try_from(result).expect("valid CoreError JsonString");
        // assert_eq!("Base for link not found", core_err.kind.to_string(),);
    }

    #[test]
    fn returns_ok_if_base_is_present() {
        let (instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result = test_zome_api_function_call(
            &context.get_dna().unwrap().name.to_string(),
            context.clone(),
            &instance,
            &context.get_wasm(&test_zome_name()).unwrap().code,
            test_link_args_bytes(String::from("test-tag")),
        );

        assert_eq!(
            call_result,
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(None))) + "\u{0}"
            ),
        );
    }

    #[test]
    fn errors_with_wrong_tag() {
        let (instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result = test_zome_api_function_call(
            &context.get_dna().unwrap().name.to_string(),
            context.clone(),
            &instance,
            &context.get_wasm(&test_zome_name()).unwrap().code,
            test_link_args_bytes(String::from("wrong-tag")),
        );

        let result = ZomeApiInternalResult::try_from(call_result)
            .expect("valid ZomeApiInternalResult JsonString");

        let core_err = CoreError::try_from(result).expect("valid CoreError JsonString");
        assert_eq!("Unknown entry type", core_err.kind.to_string(),);
    }

    #[test]
    fn works_with_linked_from_defined_link() {
        let (instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        context
            .block_on(commit_entry(test_entry_b(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result = test_zome_api_function_call(
            &context.get_dna().unwrap().name.to_string(),
            context.clone(),
            &instance,
            &context.get_wasm(&test_zome_name()).unwrap().code,
            test_link_2_args_bytes(String::from("test-tag")),
        );

        assert_eq!(
            call_result,
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(None))) + "\u{0}"
            ),
        );
    }

}
