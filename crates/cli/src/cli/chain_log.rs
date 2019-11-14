use crate::error::DefaultResult;
use colored::*;
use holochain_core::{
    agent::{
        chain_store::ChainStore,
        state::{AgentState, AgentStateSnapshot},
    },
    content_store::GetContent,
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};
use holochain_locksmith::RwLock;
use holochain_persistence_api::cas::content::Address;
use holochain_persistence_file::cas::file::FilesystemStorage;
use std::{fs, path::PathBuf};

// TODO: use system-agnostic default path
const DEFAULT_CHAIN_PATH: &str = "TODO";

pub fn chain_log(storage_path: Option<PathBuf>, instance_id: String) -> DefaultResult<()> {
    let storage_path = storage_path.ok_or_else(|| {
        format_err!("Please specify the path to CAS storage with the --path option.")
    })?;
    let cas_path = storage_path.join(instance_id).join("cas");
    let chain_store = ChainStore::new(std::sync::Arc::new(RwLock::new(
        FilesystemStorage::new(cas_path.clone()).expect("Could not create chain store"),
    )));

    let agent = chain_store
        .get_raw(&Address::from("AgentState"))?
        .ok_or("Chain does not exist or has not been initialized")
        .and_then(|snapshot_json| {
            AgentStateSnapshot::from_json_str(&snapshot_json.to_string())
                .map_err(|_| "AgentState is malformed")
        })
        .map(|snapshot| {
            let top_header = snapshot.top_chain_header().to_owned();
            AgentState::new_with_top_chain_header(
                chain_store.clone(),
                top_header.cloned(),
                Address::new(),
            )
        })
        .map_err(|err| {
            format_err!(
                "Could not display chain for '{}': {}",
                cas_path.to_string_lossy(),
                err.to_string()
            )
        })?;

    println!(
        "\nChain entries for '{}' (latest on top):\n",
        cas_path.to_string_lossy()
    );
    for ref header in agent.iter_chain() {
        let entry = chain_store
            .get(header.entry_address())
            .expect("Panic while fetching from CAS!")
            .ok_or_else(|| {
                println!(
                    "{:?} referenced in header but not found in CAS!",
                    header.entry_address(),
                )
            })
            .unwrap();
        display_header(&header, &entry);
    }

    Ok(())
}

pub fn chain_list(path: Option<PathBuf>) {
    let path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_CHAIN_PATH));
    println!("Please specify an instance ID to view its chain.");
    println!("Available instances for '{}':\n", path.to_string_lossy());
    for entry in fs::read_dir(path).unwrap() {
        let name = entry.unwrap().file_name();
        println!("- {}", name.to_string_lossy());
    }
}

fn display_header(header: &ChainHeader, entry: &Entry) {
    println!(
        "{} {}",
        header.timestamp().to_string().bright_black(),
        // format!("{:?}", header.entry_type()).blue().bold(),
        header.entry_address().to_string().yellow(),
    );
    println!("{:#?}", entry);
}
