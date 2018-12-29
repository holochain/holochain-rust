pub mod build_validation_package;
pub mod get_entry;
pub mod initialize;
pub mod validate;

#[cfg(test)]
pub mod tests {
    use crate::{
        agent::actions::commit::commit_entry,
        context::Context,
        instance::{tests::test_instance_and_context_by_name, Instance},
    };
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::ChainHeader,
        dna::{
            capabilities::{Capability, CapabilityType},
            entry_types::EntryTypeDef,
            Dna,
        },
        entry::Entry,
        json::RawString,
    };
    use std::sync::Arc;
    use test_utils::*;

    #[cfg_attr(tarpaulin, skip)]
    pub fn instance() -> (Instance, Arc<Context>) {
        instance_by_name("jane", test_dna())
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_dna() -> Dna {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "/tmp/holochain/core/src/nucleus/actions/wasm-test/target/wasm32-unknown-unknown/release/nucleus_actions_tests.wasm",
        );

        let mut dna = create_test_dna_with_cap(
            "test_zome",
            "test_cap",
            &Capability::new(CapabilityType::Public),
            &wasm,
        );

        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert("package_entry".into(), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert("package_chain_entries".into(), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert("package_chain_headers".into(), EntryTypeDef::new());
        dna.zomes
            .get_mut("test_zome")
            .unwrap()
            .entry_types
            .insert("package_chain_full".into(), EntryTypeDef::new());

        dna
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn instance_by_name(name: &str, dna: Dna) -> (Instance, Arc<Context>) {
        let (instance, context) =
            test_instance_and_context_by_name(dna, name).expect("Could not create test instance");
        let initialized_context = instance.initialize_context(context);
        (instance, initialized_context)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_entry() -> Entry {
        Entry::App("package_entry".into(), RawString::from("test value").into())
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_entries() -> Entry {
        Entry::App("package_chain_entries".into(), "test value".into())
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_headers() -> Entry {
        Entry::App("package_chain_headers".into(), "test value".into())
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_entry_package_chain_full() -> Entry {
        Entry::App("package_chain_full".into(), "test value".into())
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn commit(entry: Entry, context: &Arc<Context>) -> ChainHeader {
        let chain = context.state().unwrap().agent().chain();

        let commit_result = block_on(commit_entry(entry.clone(), None, &context.clone()));
        assert!(commit_result.is_ok());

        let top_header = context.state().unwrap().agent().top_chain_header();
        chain
            .iter(&top_header)
            .find(|ref header| *header.entry_address() == entry.address())
            .expect("Couldn't find header in chain for given entry")
    }

    // smoke test just to make sure our testing code works.
    #[test]
    pub fn can_instantiate_test_instance() {
        let (instance, _context) = instance();
        assert!(instance.state().nucleus().has_initialized());
    }

}
