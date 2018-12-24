/// Agent is the module that handles the user’s identity and source chain for every Phenotype.
///
pub mod actions;
pub mod chain_store;
pub mod state;

use crate::context::{ContextOnly, ContextStateful};
use holochain_core_types::{
    cas::content::AddressableContent, chain_header::ChainHeader, entry::Entry,
};
use std::sync::Arc;

pub fn find_chain_header(entry: &Entry, context: &Arc<ContextStateful>) -> Option<ChainHeader> {
    let chain = context.state().agent().chain();
    let top_header = context.state().agent().top_chain_header();
    chain
        .iter(&top_header)
        .find(|ref header| *header.entry_address() == entry.address())
}
