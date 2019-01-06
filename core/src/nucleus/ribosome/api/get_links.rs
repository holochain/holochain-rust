use crate::{
    network::actions::get_links::get_links,
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
};
use futures::executor::block_on;
use holochain_wasm_utils::api_serialization::get_links::{
    GetLinksArgs, GetLinksResult, LinksStatusRequestKind,
};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_links(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetLinksArgs::try_from(args_str.clone()) {
        Ok(input) => input,
        Err(_) => {
            println!(
                "invoke_get_links failed to deserialize GetLinksArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    if input.options.status_request != LinksStatusRequestKind::Live {
        runtime
            .context
            .log("get links status request other than Live not implemented!");
        return ribosome_error_code!(Unspecified);
    }

    if input.options.sources {
        runtime
            .context
            .log("get links retrieve sources not implemented!");
        return ribosome_error_code!(Unspecified);
    }

    // Get links from DHT
    let maybe_links = block_on(get_links(&runtime.context, &input.entry_address, input.tag));

    runtime.store_result(match maybe_links {
        Ok(links) => Ok(GetLinksResult::new(links)),
        Err(hc_err) => Err(hc_err),
    })
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use crate::{
        agent::actions::commit::commit_entry,
        dht::actions::add_link::add_link,
        instance::tests::{test_context_and_logger, test_instance},
        nucleus::{
            ribosome::{
                api::{tests::*, ZomeApiFunction},
                Defn,
            },
            tests::*,
        },
    };
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::Address,
        entry::{entry_type::test_app_entry_type, Entry},
        json::JsonString,
        link::Link,
    };
    use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, GetLinksOptions};
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_get_links_args_bytes(base: &Address, tag: &str) -> Vec<u8> {
        let args = GetLinksArgs {
            entry_address: base.clone(),
            tag: String::from(tag),
            options: GetLinksOptions::default(),
        };
        serde_json::to_string(&args)
            .expect("args should serialize")
            .into_bytes()
    }

    #[test]
    fn returns_list_of_links() {
        let wasm = test_zome_api_function_wasm(ZomeApiFunction::GetLinks.as_str());
        let dna = test_utils::create_test_dna_with_wasm(
            &test_zome_name(),
            &test_capability_name(),
            wasm.clone(),
        );

        let dna_name = &dna.name.to_string().clone();
        let instance = test_instance(dna).expect("Could not create test instance");

        let (context, _) = test_context_and_logger("joan");
        let initialized_context = instance.initialize_context(context);

        let mut entry_addresses: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::App(test_app_entry_type(), format!("entry{} value", i).into());
            let address = block_on(commit_entry(entry, None, &initialized_context))
                .expect("Could not commit entry for testing");
            entry_addresses.push(address);
        }

        let link1 = Link::new(&entry_addresses[0], &entry_addresses[1], "test-tag");
        let link2 = Link::new(&entry_addresses[0], &entry_addresses[2], "test-tag");

        assert!(block_on(add_link(&link1, &initialized_context)).is_ok());
        assert!(block_on(add_link(&link2, &initialized_context)).is_ok());

        let call_result = test_zome_api_function_call(
            &dna_name,
            initialized_context.clone(),
            &instance,
            &wasm,
            test_get_links_args_bytes(&entry_addresses[0], "test-tag"),
        );

        let expected_1 = JsonString::from(
            format!(
                r#"{{"ok":true,"value":"{{\"addresses\":[\"{}\",\"{}\"]}}","error":"null"}}"#,
                entry_addresses[1], entry_addresses[2]
            ) + "\u{0}",
        );

        let expected_2 = JsonString::from(
            format!(
                r#"{{"ok":true,"value":"{{\"addresses\":[\"{}\",\"{}\"]}}","error":"null"}}"#,
                entry_addresses[2], entry_addresses[1]
            ) + "\u{0}",
        );

        assert!(
            call_result == expected_1 || call_result == expected_2,
            "\n call_result = '{:?}'\n   ordering1 = '{:?}'\n   ordering2 = '{:?}'",
            call_result,
            expected_1,
            expected_2,
        );

        let call_result = test_zome_api_function_call(
            &dna_name,
            initialized_context.clone(),
            &instance,
            &wasm,
            test_get_links_args_bytes(&entry_addresses[0], "other-tag"),
        );

        assert_eq!(
            call_result,
            JsonString::from(
                String::from(r#"{"ok":true,"value":"{\"addresses\":[]}","error":"null"}"#)
                    + "\u{0}"
            ),
        );
    }

}
