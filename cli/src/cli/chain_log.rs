use crate::error::DefaultResult;
use colored::*;
use holochain_cas_implementations::cas::file::FilesystemStorage;
use holochain_core::agent::{
    chain_store::ChainStore,
    state::{AgentState, AgentStateSnapshot},
};
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};
use std::{convert::TryFrom, path::PathBuf};

pub fn chain_log(_chain_path: &PathBuf) -> DefaultResult<()> {
    let chain_path =
        PathBuf::new().join("/home/michael/.holochain/holo/storage/holo-hosting-app/cas/");
    let snapshot_json =
        include_str!("/home/michael/.holochain/holo/storage/holo-hosting-app/cas/AgentState.txt");
    let snapshot = AgentStateSnapshot::from_json_str(snapshot_json)?;
    let chain_store = ChainStore::new(std::sync::Arc::new(std::sync::RwLock::new(
        FilesystemStorage::new(chain_path).expect("could not create chain store"),
    )));
    let cas_lock = chain_store.content_storage();
    let cas = cas_lock.read().unwrap();
    let agent =
        AgentState::new_with_top_chain_header(chain_store, snapshot.top_chain_header().cloned());

    if agent.top_chain_header().is_none() {
        println!("Chain is empty.")
    } else {
        for ref header in agent.iter_chain() {
            let content = cas
                .fetch(header.entry_address())
                .expect("Panic while fetching from CAS!")
                .ok_or_else(|| {
                    println!(
                        "{:?} referenced in header but not found in CAS!",
                        header.entry_address(),
                    )
                })
                .unwrap();
            let entry = Entry::try_from(content).expect("Invalid content");
            display_header(&header, &entry);
        }
    }

    Ok(())
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
