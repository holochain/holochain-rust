use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::get_link_result::get_link_result_workflow,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I64
pub fn invoke_get_links(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetLinksArgs::try_from(args_str.clone()) {
        Ok(input) => {
            context.log(format!(
                "log/get_links: invoke_get_links called with {:?}",
                input,
            )); 
            input
        },
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_get_links failed to deserialize GetLinksArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = context.block_on(get_link_result_workflow(&context, &input));

    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    use test_utils;
    use std::sync::Arc;

    use crate::{
        context::Context,
        agent::actions::commit::commit_entry,
        dht::actions::add_link::add_link,
        instance::tests::{test_context_and_logger, test_instance},
        nucleus::ribosome::{
            api::{tests::*, ZomeApiFunction},
            Defn,
        },
    };
    use holochain_core_types::{
        cas::content::Address,
        entry::{entry_type::test_app_entry_type, Entry},
        json::{JsonString, RawString},
        link::Link,
    };
    use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_get_links_args_bytes(base: &Address, link_type: &str, tag: Option<String>) -> Vec<u8> {
        let args = GetLinksArgs {
            entry_address: base.clone(),
            link_type: String::from(link_type),
            tag: tag.into(),
            options: Default::default(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    fn add_test_entries(initialized_context: Arc<Context>) -> Vec<Address> {
        let mut entry_addresses: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::App(
                test_app_entry_type(),
                JsonString::from(RawString::from(format!("entry{} value", i))),
            );
            let address = initialized_context
                .block_on(commit_entry(entry, None, &initialized_context))
                .expect("Could not commit entry for testing");
            entry_addresses.push(address);
        }
        entry_addresses
    }

    fn initialize_context() -> Arc<Context> {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::GetLinks.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        let netname = Some("returns_list_of_links");
        let instance = test_instance(dna, netname).expect("Could not create test instance");

        let (context, _) = test_context_and_logger("joan", netname);
        instance.initialize_context(context)
    }

    fn add_links(initialized_context: Arc<Context>, links: Vec<Link>) {

        links.iter().for_each(|link| {
            assert!(initialized_context
                .block_on(add_link(&link, &initialized_context))
                .is_ok());
        });
    }

    fn get_links(initialized_context: Arc<Context>, base: &Address, link_type: &str, tag: Option<String>) -> JsonString {
        test_zome_api_function_call(
            initialized_context.clone(),
            test_get_links_args_bytes(&base, link_type, tag),
        )
    }

    #[test]
    fn returns_list_of_links() {

        // setup the instance and links
        let initialized_context = initialize_context();
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(&entry_addresses[0], &entry_addresses[1], "test-type", "test-tag"),
            Link::new(&entry_addresses[0], &entry_addresses[2], "test-type", "test-tag"),
        ];
        add_links(initialized_context.clone(), links);
        
        // calling get_links returns both links in some order
        let call_result = get_links(initialized_context.clone(), &entry_addresses[0], "test-type", Some("test-tag".into()));
        let expected_1 = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[]}},{{\"address\":\"{}\",\"headers\":[]}}]}}","error":"null"}}"#,
                entry_addresses[1], entry_addresses[2]
            ) + "\u{0}"),
        );
        let expected_2 = JsonString::from_json(
            &(format!(
               r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[]}},{{\"address\":\"{}\",\"headers\":[]}}]}}","error":"null"}}"#,
                entry_addresses[2], entry_addresses[1]
            ) + "\u{0}"),
        );
        assert!(
            call_result == expected_1 || call_result == expected_2,
            "\n call_result = '{:?}'\n   ordering1 = '{:?}'\n   ordering2 = '{:?}'",
            call_result,
            expected_1,
            expected_2,
        );

        // calling get_links with another non-existent type returns nothing
        let call_result = get_links(initialized_context.clone(), &entry_addresses[0], "other-type", Some("test-tag".into()));
        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(r#"{"ok":true,"value":"{\"links\":[]}","error":"null"}"#,)
                    + "\u{0}")
            ),
        );

        // calling get_links with another non-existent tag returns nothing
        let call_result = get_links(initialized_context.clone(), &entry_addresses[0], "test-type", Some("other-tag".into()));
        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(r#"{"ok":true,"value":"{\"links\":[]}","error":"null"}"#,)
                    + "\u{0}")
            ),
        );
    }

    #[test]
    fn test_with_same_target_and_tag_dedup() {
        let initialized_context = initialize_context();
        let entry_addresses = add_test_entries(initialized_context.clone());
        // links have same tag, same base and same tag. Are the same
        let links = vec![
            Link::new(&entry_addresses[0], &entry_addresses[1], "test-type", "test-tag"),
            Link::new(&entry_addresses[0], &entry_addresses[1], "test-type", "test-tag"),
        ];
        add_links(initialized_context.clone(), links);
        let call_result = get_links(initialized_context.clone(), &entry_addresses[0], "test-type", Some("test-tag".into()));
        let expected = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "?",
            ) + "\u{0}"),
        );
        assert_eq!(
            call_result,
            expected,
        );
    }

    #[test]
    fn test_with_same_target_different_tag_dont_dedup() {
        let initialized_context = initialize_context();
        let entry_addresses = add_test_entries(initialized_context.clone());
        // same target and type, different tag
        let links = vec![
            Link::new(&entry_addresses[0], &entry_addresses[1], "test-type", "test-tag1"),
            Link::new(&entry_addresses[0], &entry_addresses[1], "test-type", "test-tag2"),
        ];
        add_links(initialized_context.clone(), links);
        let call_result = get_links(initialized_context.clone(), &entry_addresses[0], "test-type", None);
        let expected = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "?", entry_addresses[1], "?",
            ) + "\u{0}"),
        );
        assert_eq!(
            call_result,
            expected,
        );
    }
}
