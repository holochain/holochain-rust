use crate::{
    action::{ActionWrapper, GetEntryKey},
    context::Context,
    network::state::NetworkState,
};
use holochain_core_types::{cas::content::Address, entry::EntryWithMeta, error::HolochainError};
use holochain_net::connection::json_protocol::FetchEntryResultData;
use std::sync::Arc;

fn reduce_handle_get_result_inner(
    network_state: &mut NetworkState,
    dht_data: &FetchEntryResultData,
) -> Result<Option<EntryWithMeta>, HolochainError> {
    network_state.initialized()?;

    let res = serde_json::from_str(&serde_json::to_string(&dht_data.entry_content).unwrap());
    if let Err(_) = res {
        return Err(HolochainError::ErrorGeneric(
            "Failed to deserialize EntryWithMeta from HandleFetchResult action argument"
                .to_string(),
        ));
    }
    Ok(res.unwrap())
}

pub fn reduce_handle_get_result(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let dht_data = unwrap_to!(action => crate::action::Action::HandleFetchResult);

    let result = reduce_handle_get_result_inner(network_state, dht_data);

    let key = GetEntryKey {
        address: Address::from(dht_data.entry_address.clone()),
        id: dht_data.request_id.clone(),
    };

    network_state
        .get_entry_with_meta_results
        .insert(key, Some(result));
}
