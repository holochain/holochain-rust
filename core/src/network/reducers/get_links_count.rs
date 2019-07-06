use crate::{
    action::{ActionWrapper, GetLinksKey},
    network::{query::NetworkQuery, reducers::send, state::NetworkState},
    state::State,
};

use holochain_core_types::{crud_status::CrudStatus, error::HolochainError};
use holochain_json_api::json::JsonString;
use holochain_net::connection::json_protocol::{JsonProtocol, QueryEntryData};
use holochain_persistence_api::hash::HashString;

fn reduce_get_links_inner(
    network_state: &mut NetworkState,
    key: &GetLinksKey,
    crud_status: Option<CrudStatus>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let query_json: JsonString =
        NetworkQuery::GetLinksCount(key.link_type.clone(), key.tag.clone(), crud_status).into();
    send(
        network_state,
        JsonProtocol::QueryEntry(QueryEntryData {
            requester_agent_id: network_state.agent_id.clone().unwrap().into(),
            request_id: key.id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            entry_address: HashString::from(key.base_address.clone()),
            query: query_json.to_string().into_bytes(),
        }),
    )
}
pub fn reduce_get_links_count(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (key, crud) = unwrap_to!(action => crate::action::Action::GetLinksCount);

    let result = match reduce_get_links_inner(network_state, &key, crud.clone()) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_links_results_count
        .insert(key.clone(), result);
}
