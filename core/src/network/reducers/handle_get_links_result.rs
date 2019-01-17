use crate::{action::ActionWrapper, context::Context, network::state::NetworkState};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use holochain_net_connection::json_protocol::DhtMetaData;
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    dht_meta_data: &DhtMetaData,
) -> Result<Vec<Address>, HolochainError> {
    network_state.initialized()?;

    let res = serde_json::from_str(&serde_json::to_string(&dht_meta_data.content).unwrap());
    if let Err(_) = res {
        return Err(HolochainError::ErrorGeneric(
            "Failed to deserialize Vec<Address> from HandleGetLinkResult DhtMetaData content"
                .to_string(),
        ));
    }
    Ok(res.unwrap())
}

pub fn reduce_handle_get_links_result(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (dht_meta_data, tag) = unwrap_to!(action => crate::action::Action::HandleGetLinksResult);

    let result = inner(network_state, dht_meta_data);

    network_state.get_links_results.insert(
        (Address::from(dht_meta_data.address.clone()), tag.clone()),
        Some(result),
    );
}
