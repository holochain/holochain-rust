use boolinator::*;
use crate::{
    action::ActionWrapper,
    context::Context,
    network::state::NetworkState,
};
use holochain_core_types::{
    cas::content::Address,
    entry::Entry,
    error::HolochainError,
};
use holochain_net_connection::{
    protocol_wrapper::DhtData,
};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    dht_data: &DhtData,
) -> Result<Option<Entry>, HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
        .ok_or("Network not initialized".to_string())?;

    let entry: Option<Entry> =
        serde_json::from_str(&serde_json::to_string(&dht_data.content).unwrap())?;

    Ok(entry)
}

pub fn reduce_handle_get_result(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let dht_data = unwrap_to!(action => crate::action::Action::HandleGetResult);

    let result = inner(network_state, dht_data);

    network_state.get_entry_results.insert(
        Address::from(dht_data.address.clone()),
        Some(result),
    );
}
