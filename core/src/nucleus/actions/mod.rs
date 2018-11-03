pub mod build_validation_package;
pub mod get_entry;
pub mod initialize;
pub mod validate;

#[cfg(test)]
pub mod tests {
    use agent::actions::commit::commit_entry;
    use context::Context;
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent, chain_header::ChainHeader, entry::Entry,
        entry_type::EntryType, json::JsonString,
    };
    use holochain_dna::zome::{capabilities::Capability, entry_types::EntryTypeDef};
    use instance::{tests::test_instance_and_context, Instance};
    use std::sync::Arc;
    use test_utils::*;

    #[cfg_attr(tarpaulin, skip)]
    pub fn instance() -> (Instance, Arc<Context>) {
        // Setup the holochain instance
        let wasm =
            create_wasm_from_file("src/nucleus/actions/wasm-test/target/wasm32-unknown-unknown/release/nucleus_actions_tests.wasm");

        let mut dna = create_test_dna_with_cap("test_zome", "test_cap", &Capability::new(), &wasm);

        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert(String::from("package_entry"), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert(String::from("package_chain_entries"), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert(String::from("package_chain_headers"), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert(String::from("package_chain_full"), EntryTypeDef::new());

        let (instance, context) =
            test_instance_and_context(dna).expect("Could not create test instance");
        let initialized_context = instance.initialize_context(context);

        (instance, initialized_context)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_entry() -> Entry {
        Entry::new(
            EntryType::App(String::from("package_entry")),
            JsonString::from("test value"),
        )
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_entries() -> Entry {
        Entry::new(
            EntryType::App(String::from("package_chain_entries")),
            JsonString::from("test value"),
        )
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_headers() -> Entry {
        Entry::new(
            EntryType::App(String::from("package_chain_headers")),
            JsonString::from("test value"),
        )
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_full() -> Entry {
        Entry::new(
            EntryType::App(String::from("package_chain_full")),
            JsonString::from("test value"),
        )
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn commit(entry: Entry, context: &Arc<Context>) -> ChainHeader {
        let chain = context.state().unwrap().agent().chain();

        let commit_result = block_on(commit_entry(
            entry.clone(),
            &context.clone().action_channel,
            &context.clone(),
        ));
        assert!(commit_result.is_ok());

        let top_header = context.state().unwrap().agent().top_chain_header();
        chain
            .iter(&top_header)
            .find(|ref header| *header.entry_address() == entry.address())
            .expect("Couldn't find header in chain for given entry")
    }
}
