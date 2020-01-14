use crate::{
    ribosome::{api::ZomeApiResult, runtime::Runtime},
    workflows::author_entry::author_entry,
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    link::{link_data::LinkData, LinkActionKind},
};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_utils::api_serialization::link_entries::LinkEntriesArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::LinkEntries function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: LinkEntriesArgs
pub fn invoke_link_entries(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match LinkEntriesArgs::try_from(args_str.clone()) {
        Ok(entry_input) => entry_input,
        // Exit on error
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };
    let top_chain_header_option = context
        .state()
        .expect("Couldn't get state in invoke_linke_entries")
        .agent()
        .top_chain_header();

    let top_chain_header = match top_chain_header_option {
        Some(top_chain) => top_chain,
        None => {
            log_error!(
                context,
                "zome: invoke_link_entries failed to deserialize LinkEntriesArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let link = input.to_link();
    let link_add = LinkData::from_link(
        &link,
        LinkActionKind::ADD,
        top_chain_header,
        context.agent_id.clone(),
    );
    let entry = Entry::LinkAdd(link_add);

    // Wait for future to be resolved
    // This is where the link entry actually gets created.
    let result: Result<Address, HolochainError> = context
        .block_on(author_entry(&entry, None, &context, &vec![]))
        .map(|_| entry.address());

    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    use test_utils;

    use crate::{
        agent::actions::commit::commit_entry,
        context::Context,
        instance::{tests::test_instance_and_context, Instance},
        nucleus::ribosome::{
            api::{tests::*, ZomeApiFunction},
            Defn,
        },
    };
    use holochain_core_types::{
        entry::{test_entry, Entry},
        error::{CoreError, ZomeApiInternalResult},
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::AddressableContent;
    use holochain_wasm_utils::api_serialization::link_entries::*;

    use serde_json;
    use std::{convert::TryFrom, sync::Arc};

    pub fn test_entry_b() -> Entry {
        Entry::App("testEntryTypeB".into(), "test".into())
    }

    /// dummy link_entries args from standard test entry
    pub fn test_link_args_bytes(link_type: String, tag: String) -> Vec<u8> {
        let entry = test_entry();

        let args = LinkEntriesArgs {
            base: entry.address(),
            target: entry.address(),
            link_type,
            tag,
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    pub fn test_link_2_args_bytes(link_type: String, tag: String) -> Vec<u8> {
        let base = test_entry();
        let target = test_entry_b();

        let args = LinkEntriesArgs {
            base: base.address(),
            target: target.address(),
            link_type,
            tag,
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    /// dummy commit args from standard test entry
    pub fn test_commit_args_bytes() -> Vec<u8> {
        JsonString::from(test_entry()).to_bytes()
    }

    fn create_test_instance_with_name(netname: Option<&str>) -> (Instance, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::LinkEntries.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        test_instance_and_context(dna, netname).expect("Could not create test instance")
    }
    fn create_test_instance() -> (Instance, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::LinkEntries.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        let netname = format!("create_test_instance-{}", snowflake::ProcessUniqueId::new());

        test_instance_and_context(dna, Some(netname.as_str()))
            .expect(format!("Could not create test instance for netname: {}", netname).as_str())
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    #[cfg(not(windows))]
    fn errors_if_base_is_not_present_test() {
        // let (call_result, _) = test_zome_api_function(
        //     ZomeApiFunction::LinkEntries.as_str(),
        //     test_link_args_bytes(String::from("test-link")),
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
        let (_instance, context) =
            create_test_instance_with_name(Some("returns_ok_if_base_present"));

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result_json = test_zome_api_function_call(
            context.clone(),
            test_link_args_bytes("test-link".into(), "test-tag".into()),
        );

        let call_result = ZomeApiInternalResult::try_from(call_result_json);

        assert!(call_result.is_ok())
    }

    #[test]
    fn errors_with_wrong_type() {
        let (_instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result = test_zome_api_function_call(
            context.clone(),
            test_link_args_bytes("wrong-link".into(), "test-tag".into()),
        );

        let result = ZomeApiInternalResult::try_from(call_result)
            .expect("valid ZomeApiInternalResult JsonString");

        let core_err = CoreError::try_from(result).expect("valid CoreError JsonString");
        assert_eq!("Unknown entry type", core_err.kind.to_string(),);
    }

    #[test]
    fn works_with_linked_from_defined_link() {
        let (_instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        context
            .block_on(commit_entry(test_entry_b(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result_json = test_zome_api_function_call(
            context.clone(),
            test_link_2_args_bytes("test-link".into(), "test-tag".into()),
        );

        let call_result = ZomeApiInternalResult::try_from(call_result_json);

        assert!(call_result.is_ok())
    }

    #[test]
    fn test_different_tags_produces_different_hashes() {
        let (_instance, context) = create_test_instance();

        context
            .block_on(commit_entry(test_entry(), None, &context))
            .expect("Could not commit entry for testing");

        let call_result1 = test_zome_api_function_call(
            context.clone(),
            test_link_args_bytes("test-link".into(), "test-tag1".into()),
        );
        let call_result2 = test_zome_api_function_call(
            context.clone(),
            test_link_args_bytes("test-link".into(), "test-tag2".into()),
        );

        let result1: JsonString = ZomeApiInternalResult::success(call_result1).into();
        let result2: JsonString = ZomeApiInternalResult::success(call_result2).into();

        assert_ne!(result1, result2);
    }
}
