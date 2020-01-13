use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::get_link_result::get_link_result_workflow,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code.
/// args: [0] encoded MemoryAllocation as u64
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I64
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_get_links(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetLinksArgs::try_from(args_str.clone()) {
        Ok(input) => {
            log_debug!(
                context,
                "zome/get_links: invoke_get_links called with {:?}",
                input,
            );
            input
        }
        Err(_) => {
            log_error!(
                context,
                "zome/get_links: invoke_get_links failed to deserialize GetLinksArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = context.block_on(get_link_result_workflow(&context, &input));

    runtime.store_result(result)
}

#[cfg(test)]
pub mod tests {
    use crate::instance::Instance;
    use std::sync::Arc;
    use test_utils;

    use crate::{
        agent::actions::commit::commit_entry,
        context::Context,
        dht::actions::hold_aspect::hold_aspect,
        instance::tests::{
            test_context_and_logger_with_in_memory_network, test_instance_and_context,
        },
        nucleus::ribosome::{
            api::{tests::*, ZomeApiFunction},
            Defn,
        },
    };
    use holochain_core_types::{
        agent::test_agent_id,
        chain_header::test_chain_header,
        entry::{entry_type::test_app_entry_type, Entry},
        link::{link_data::LinkData, Link, LinkMatch},
        network::entry_aspect::EntryAspect,
    };
    use holochain_json_api::json::{JsonString, RawString};
    use holochain_persistence_api::cas::content::Address;
    use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_get_links_args_bytes(
        base: &Address,
        link_type: LinkMatch<String>,
        tag: LinkMatch<String>,
    ) -> Vec<u8> {
        let args = GetLinksArgs {
            entry_address: base.clone(),
            link_type: link_type.to_regex_string().unwrap(),
            tag: tag.to_regex_string().unwrap(),
            options: Default::default(),
        };
        println!("GetLinksArgs: {:?}", args);
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

    fn initialize_context(netname: &str) -> (Instance, Arc<Context>) {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::GetLinks.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());
        let netname2 = format!("{}-2", netname);
        let (instance, _) =
            test_instance_and_context(dna, Some(netname)).expect("Could not create test instance");
        let (context, _) =
            test_context_and_logger_with_in_memory_network("joan", Some(netname2.as_str()));
        let arc_context = instance.initialize_context(context);
        (instance, arc_context)
    }

    pub fn add_links(initialized_context: Arc<Context>, links: Vec<Link>) {
        links.iter().for_each(|link| {
            println!("adding link {:?}", link);
            assert!(initialized_context //commit the AddLink entry first
                .block_on(commit_entry(
                    link.add_entry(test_chain_header(), test_agent_id()),
                    None,
                    &initialized_context
                ))
                .is_ok());
            assert!(initialized_context
                .block_on(hold_aspect(
                    EntryAspect::LinkAdd(
                        LinkData::add_from_link(link, test_chain_header(), test_agent_id()),
                        test_chain_header(),
                    ),
                    initialized_context.clone()
                ))
                .is_ok());
            println!("added link {:?}", link);
        });
    }

    pub fn get_links(
        initialized_context: Arc<Context>,
        base: &Address,
        link_type: LinkMatch<String>,
        tag: LinkMatch<String>,
    ) -> JsonString {
        test_zome_api_function_call(
            initialized_context.clone(),
            test_get_links_args_bytes(&base, link_type, tag),
        )
    }

    // TODO do this for all crate tests somehow
    fn enable_logging_for_test() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "trace");
        }
        let _ = env_logger::builder()
            .default_format_timestamp(false)
            .default_format_module_path(false)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn returns_list_of_links() {
        enable_logging_for_test();
        // setup the instance and links
        let (_instance, initialized_context) = initialize_context("returns_list_of_links");
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[2],
                "test-type",
                "test-tag",
            ),
        ];
        add_links(initialized_context.clone(), links);

        // calling get_links returns both links in some order
        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("test-type".into()),
            LinkMatch::Exactly("test-tag".into()),
        );
        let expected_1 = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "test-tag", entry_addresses[2], "test-tag",
            ) + "\u{0}"),
        );
        let expected_2 = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[2], "test-tag", entry_addresses[1], "test-tag",
            ) + "\u{0}"),
        );
        assert!(
            call_result == expected_1 || call_result == expected_2,
            "\n call_result = '{:?}'\n   ordering1 = '{:?}'\n   ordering2 = '{:?}'",
            call_result,
            expected_1,
            expected_2,
        );
    }

    #[test]
    fn get_links_with_non_existent_type_returns_nothing() {
        let (_instance, initialized_context) =
            initialize_context("get_links_with_non_existent_type_returns_nothing");
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[2],
                "test-type",
                "test-tag",
            ),
        ];
        add_links(initialized_context.clone(), links);

        // calling get_links with another non-existent type returns nothing
        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("other-type".into()),
            LinkMatch::Exactly("test-tag".into()),
        );
        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(r#"{"ok":true,"value":"{\"links\":[]}","error":"null"}"#,)
                    + "\u{0}")
            ),
        );
    }

    #[test]
    fn get_links_with_non_existent_tag_returns_nothing() {
        let (_instance, initialized_context) =
            initialize_context("get_links_with_non_existent_tag_returns_nothing");
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[2],
                "test-type",
                "test-tag",
            ),
        ];
        add_links(initialized_context.clone(), links);

        // calling get_links with another non-existent tag returns nothing
        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("test-type".into()),
            LinkMatch::Exactly("other-tag".into()),
        );
        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(r#"{"ok":true,"value":"{\"links\":[]}","error":"null"}"#,)
                    + "\u{0}")
            ),
        );
    }

    #[test]
    fn can_get_all_links_of_any_tag_or_type() {
        // setup the instance and links
        let (_instance, initialized_context) =
            initialize_context("can_get_all_links_of_any_tag_or_type");
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type1",
                "test-tag1",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[2],
                "test-type2",
                "test-tag2",
            ),
        ];
        add_links(initialized_context.clone(), links);

        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Any,
            LinkMatch::Any,
        );
        let expected_1 = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "test-tag1", entry_addresses[2], "test-tag2",
            ) + "\u{0}"),
        );
        let expected_2 = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[2], "test-tag2", entry_addresses[1], "test-tag1",
            ) + "\u{0}"),
        );
        assert!(
            call_result == expected_1 || call_result == expected_2,
            "\n call_result = '{:?}'\n   ordering1 = '{:?}'\n   ordering2 = '{:?}'",
            call_result,
            expected_1,
            expected_2,
        );
    }

    #[test]
    fn get_links_with_exact_tag_match_returns_only_that_link() {
        let (_instance, initialized_context) =
            initialize_context("get_links_with_exact_tag_match_returns_only_that");
        let entry_addresses = add_test_entries(initialized_context.clone());
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag1",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag2",
            ),
        ];
        add_links(initialized_context.clone(), links);

        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("test-type".into()),
            LinkMatch::Exactly("test-tag1".into()),
        );
        let expected = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "test-tag1",
            ) + "\u{0}"),
        );
        assert_eq!(call_result, expected,);
    }

    #[test]
    fn test_with_same_target_and_tag_dedup() {
        let (_instance, initialized_context) =
            initialize_context("test_with_same_target_and_tag_dedup");
        let entry_addresses = add_test_entries(initialized_context.clone());
        // links have same tag, same base and same tag. Are the same
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag",
            ),
        ];
        add_links(initialized_context.clone(), links);
        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("test-type".into()),
            LinkMatch::Exactly("test-tag".into()),
        );
        let expected = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "test-tag",
            ) + "\u{0}"),
        );
        assert_eq!(call_result, expected,);
    }

    #[test]
    fn test_with_same_target_different_tag_dont_dedup() {
        let (_instance, initialized_context) =
            initialize_context("test_with_same_target_different_tag_dont_dedup");
        let entry_addresses = add_test_entries(initialized_context.clone());
        // same target and type, different tag
        let links = vec![
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag1",
            ),
            Link::new(
                &entry_addresses[0],
                &entry_addresses[1],
                "test-type",
                "test-tag2",
            ),
        ];
        add_links(initialized_context.clone(), links);
        let call_result = get_links(
            initialized_context.clone(),
            &entry_addresses[0],
            LinkMatch::Exactly("test-type".into()),
            LinkMatch::Any,
        );
        let expected = JsonString::from_json(
            &(format!(
                r#"{{"ok":true,"value":"{{\"links\":[{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}},{{\"address\":\"{}\",\"headers\":[],\"tag\":\"{}\",\"status\":\"live\"}}]}}","error":"null"}}"#,
                entry_addresses[1], "test-tag1", entry_addresses[1], "test-tag2",
            ) + "\u{0}"),
        );
        assert_eq!(call_result, expected,);
    }
}
