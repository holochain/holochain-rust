use holochain_core_types::cas::content::Address;
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use nucleus::ribosome::{Runtime, api::ZomeApiResult};
use serde_json;
use wasmi::{RuntimeArgs, RuntimeValue};

/// ZomeApiFunction::GetLinks function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: GetLinksArgs
/// Returns an HcApiReturnCode as I32
pub fn invoke_get_links(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: GetLinksArgs = match serde_json::from_str(&args_str) {
        Ok(input) => input,
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };
    // Get links from DHT
    let maybe_links = runtime
        .context
        .state()
        .unwrap()
        .dht()
        .get_links(input.entry_address, input.tag);
    // Write result in wasm memory
    match maybe_links {
        Err(hc_err) => runtime.store_as_json(core_error!(hc_err)),
        Ok(links) => {
            let vec_links = links
                .iter()
                .map(|eav| eav.value())
                .collect::<Vec<Address>>();
            runtime.store_as_json(vec_links)
        },
    }
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use agent::actions::commit::commit_entry;
    use dht::actions::add_link::add_link;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::Address, entry::Entry, entry_type::test_entry_type, links_entry::Link,
    };
    use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
    use instance::tests::{test_context_and_logger, test_instance};
    use nucleus::ribosome::{
        api::{tests::*, ZomeApiFunction},
        Defn,
    };
    use serde_json;

    /// dummy link_entries args from standard test entry
    pub fn test_get_links_args_bytes(base: &Address, tag: &str) -> Vec<u8> {
        let args = GetLinksArgs {
            entry_address: base.clone(),
            tag: String::from(tag),
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
            &test_capability(),
            wasm.clone(),
        );

        let dna_name = &dna.name.to_string().clone();
        let instance = test_instance(dna).expect("Could not create test instance");

        let (context, _) = test_context_and_logger("joan");
        let initialized_context = instance.initialize_context(context);

        let mut entry_hashes: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::new(&test_entry_type(), &format!("entry{} value", i));
            let hash = block_on(commit_entry(
                entry,
                &initialized_context.action_channel.clone(),
                &initialized_context,
            )).expect("Could not commit entry for testing");
            entry_hashes.push(hash);
        }

        let link1 = Link::new(&entry_hashes[0], &entry_hashes[1], "test-tag");
        let link2 = Link::new(&entry_hashes[0], &entry_hashes[2], "test-tag");

        assert!(block_on(add_link(&link1, &initialized_context)).is_ok());
        assert!(block_on(add_link(&link2, &initialized_context)).is_ok());

        let call_result = test_zome_api_function_call(
            &dna_name,
            initialized_context.clone(),
            &instance,
            &wasm,
            test_get_links_args_bytes(&entry_hashes[0], "test-tag"),
        );
        println!("call_result = '{:?}'", call_result);
        let ordering1= format!(r#"["{}","{}"]"#, entry_hashes[1], entry_hashes[2]) + "\u{0}";
        let ordering2= format!(r#"["{}","{}"]"#, entry_hashes[2], entry_hashes[1]) + "\u{0}";

        assert!(call_result == ordering1 || call_result == ordering2);

        let call_result = test_zome_api_function_call(
            &dna_name,
            initialized_context.clone(),
            &instance,
            &wasm,
            test_get_links_args_bytes(&entry_hashes[0], "other-tag"),
        );

        assert_eq!("[]\u{0}", call_result);
    }

}
