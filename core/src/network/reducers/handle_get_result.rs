use crate::{
    action::{ActionWrapper, GetEntryKey},
    network::state::NetworkState,
    state::State,
};
use lib3h_persistence_api::{
    cas::content::Address
};

use holochain_core_types::{
    entry::EntryWithMetaAndHeader, error::HolochainError,
};
use holochain_net::connection::json_protocol::FetchEntryResultData;

fn reduce_handle_get_result_inner(
    network_state: &mut NetworkState,
    dht_data: &FetchEntryResultData,
) -> Result<Option<EntryWithMetaAndHeader>, HolochainError> {
    network_state.initialized()?;
    let content = serde_json::to_string(&dht_data.entry_content).map_err(|_| {
        HolochainError::ErrorGeneric("Could not serialize entry content".to_string())
    })?;

    let entry_with_meta = serde_json::from_str(&content).map_err(|_| {
        HolochainError::ErrorGeneric(
            "Failed to deserialize EntryWithMeta from HandleFetchResult action argument"
                .to_string(),
        )
    })?;

    Ok(entry_with_meta)
}

pub fn reduce_handle_get_result(
    network_state: &mut NetworkState,
    _root_state: &State,
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
