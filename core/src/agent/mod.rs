/// Agent is the module that handles the user’s identity and source chain for every Phenotype.
///
pub mod actions;
pub mod chain_store;
pub mod state;

use crate::state::StateWrapper;
use holochain_core_types::{chain_header::ChainHeader, entry::Entry};

use holochain_persistence_api::cas::content::AddressableContent;

pub fn find_chain_header(entry: &Entry, state: &StateWrapper) -> Option<ChainHeader> {
    let chain = state.agent().chain_store();
    let top_header = state.agent().top_chain_header();
    chain
        .iter(&top_header)
        .find(|ref header| *header.entry_address() == entry.address())
}
