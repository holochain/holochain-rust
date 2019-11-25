pub mod build_validation_package;
pub mod call_init;
pub mod call_zome_function;
pub mod get_entry;
pub mod initialize;
pub mod run_validation_callback;
pub mod trace_invoke_hdk_function;
pub mod trace_return_hdk_function;

#[cfg(test)]
pub mod tests {
    use crate::{
        agent::actions::commit::commit_entry,
        context::Context,
        instance::{
            tests::{
                test_instance_and_context_by_name,
                test_instance_and_context_with_memory_network_nodes,
            },
            Instance,
        },
    };
    use holochain_core_types::{
        chain_header::ChainHeader,
        dna::{entry_types::EntryTypeDef, Dna},
        entry::Entry,
    };
    use holochain_json_api::json::RawString;
    use holochain_persistence_api::cas::content::AddressableContent;

    use holochain_wasm_utils::wasm_target_dir;
    use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
    use test_utils::*;

    #[cfg_attr(tarpaulin, skip)]
    pub fn instance(network_name: Option<&str>) -> (Instance, Arc<Context>) {
        instance_by_name("jane", test_dna(), network_name)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_dna() -> Dna {
        // Setup the holochain instance
        let target_path: PathBuf = [
            String::from("src"),
            String::from("nucleus"),
            String::from("actions"),
            String::from("wasm-test"),
        ]
        .iter()
        .collect();
        let target_dir = wasm_target_dir(&String::from("core").into(), &target_path);
        let mut wasm_path = PathBuf::new();
        let wasm_path_component: PathBuf = [
            "wasm32-unknown-unknown",
            "release",
            "nucleus_actions_tests.wasm",
        ]
        .iter()
        .collect();
        wasm_path.push(target_dir);
        wasm_path.push(wasm_path_component);

        let wasm = create_wasm_from_file(&wasm_path);

        let defs = (Vec::new(), BTreeMap::new());
        let mut dna = create_test_dna_with_defs("test_zome", defs, &wasm);

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
    pub fn instance_by_name(
        name: &str,
        dna: Dna,
        network_name: Option<&str>,
    ) -> (Instance, Arc<Context>) {
        let (instance, context) = test_instance_and_context_by_name(dna, name, network_name)
            .expect("Could not create test instance");
        let initialized_context = instance.initialize_context(context);
        (instance, initialized_context)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn instance_with_bootstrap_nodes(
        name: &str,
        dna: Dna,
        network_name: Option<&str>,
    ) -> (Instance, Arc<Context>) {
        let (instance, context) =
            test_instance_and_context_with_memory_network_nodes(dna, name, network_name)
                .expect("Could not create test instance");
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
        let chain = context.state().unwrap().agent().chain_store();

        let commit_result = context.block_on(commit_entry(entry.clone(), None, &context.clone()));
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
        let (instance, _context) = instance(None);
        assert!(instance.state().nucleus().has_initialized());
    }
}
