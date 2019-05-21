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
        Ok(input) => input,
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

    use crate::{
        agent::actions::commit::commit_entry,
        dht::actions::add_link::add_link,
        instance::tests::{test_context_and_logger, test_instance},
        nucleus::ribosome::{
            api::{tests::*, ZomeApiFunction},
            Defn,
        },
    };
    use holochain_core_types::{
        agent::test_agent_id,
        cas::content::Address,
        entry::{entry_type::test_app_entry_type, Entry},
        json::{JsonString, RawString},
        link::{link_data::LinkData, Link},
    };
    use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_get_links_args_bytes(base: &Address, tag: &str) -> Vec<u8> {
        let args = GetLinksArgs {
            entry_address: base.clone(),
            tag: String::from(tag),
            options: Default::default(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[test]
    fn returns_list_of_links() {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::GetLinks.as_str());
        let dna = test_utils::create_test_dna_with_wasm(&test_zome_name(), wasm.clone());

        let netname = Some("returns_list_of_links");
        let instance = test_instance(dna, netname).expect("Could not create test instance");

        let (context, _) = test_context_and_logger("joan", netname);
        let initialized_context = instance.initialize_context(context);

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

        let link1 = Link::new(&entry_addresses[0], &entry_addresses[1], "test-tag");
        let link2 = Link::new(&entry_addresses[0], &entry_addresses[2], "test-tag");
        let link_entry_1 = Entry::LinkAdd(LinkData::new_add(
            &entry_addresses[0],
            &entry_addresses[1].clone(),
            "test-tag",
            0,
            test_agent_id(),
        ));
        initialized_context
            .block_on(commit_entry(
                link_entry_1.clone(),
                None,
                &initialized_context,
            ))
            .expect("Could not commit link");
        let link_entry_2 = Entry::LinkAdd(LinkData::new_add(
            &entry_addresses[0],
            &entry_addresses[2].clone(),
            "test-tag",
            0,
            test_agent_id(),
        ));
        initialized_context
            .block_on(commit_entry(
                link_entry_2.clone(),
                None,
                &initialized_context,
            ))
            .expect("Could not commit link");
        assert!(initialized_context
            .block_on(add_link(&link_entry_1, &link1, &initialized_context))
            .is_ok());
        assert!(initialized_context
            .block_on(add_link(&link_entry_2, &link2, &initialized_context))
            .is_ok());

        let call_result = test_zome_api_function_call(
            initialized_context.clone(),
            test_get_links_args_bytes(&entry_addresses[0], "test-tag"),
        );

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

        let call_result = test_zome_api_function_call(
            initialized_context.clone(),
            test_get_links_args_bytes(&entry_addresses[0], "other-tag"),
        );

        assert_eq!(
            call_result,
            JsonString::from_json(
                &(String::from(r#"{"ok":true,"value":"{\"links\":[]}","error":"null"}"#,)
                    + "\u{0}")
            ),
        );
    }

}
