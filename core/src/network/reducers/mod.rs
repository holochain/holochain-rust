pub mod get_validation_package;
pub mod handle_custom_send_response;
pub mod handle_get_result;
pub mod handle_get_validation_package;
pub mod init;
pub mod publish;
pub mod publish_header_entry;
pub mod query;
pub mod resolve_direct_connection;
pub mod respond_authoring_list;
pub mod respond_fetch;
pub mod respond_gossip_list;
pub mod respond_query;
pub mod send_direct_message;
pub mod shutdown;

use crate::{
    action::{Action, ActionWrapper, NetworkReduceFn},
    network::{
        direct_message::DirectMessage,
        reducers::{
            get_validation_package::reduce_get_validation_package,
            handle_custom_send_response::reduce_handle_custom_send_response,
            handle_get_result::reduce_handle_get_result,
            handle_get_validation_package::reduce_handle_get_validation_package,
            init::reduce_init,
            publish::reduce_publish,
            publish_header_entry::reduce_publish_header_entry,
            query::{reduce_query, reduce_query_timeout},
            resolve_direct_connection::reduce_resolve_direct_connection,
            respond_authoring_list::reduce_respond_authoring_list,
            respond_fetch::reduce_respond_fetch_data,
            respond_gossip_list::reduce_respond_gossip_list,
            respond_query::reduce_respond_query,
            send_direct_message::{reduce_send_direct_message, reduce_send_direct_message_timeout},
            shutdown::reduce_shutdown,
        },
        state::NetworkState,
    },
    state::State,
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use holochain_net::connection::net_connection::NetSend;

use lib3h_protocol::{data_types::DirectMessageData, protocol_client::Lib3hClientProtocol};

use holochain_persistence_api::cas::content::Address;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snowflake::ProcessUniqueId;
use std::sync::Arc;

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NetworkReduceFn> {
    match action_wrapper.action() {
        Action::Query(_) => Some(reduce_query),
        Action::QueryTimeout(_) => Some(reduce_query_timeout),
        Action::GetValidationPackage(_) => Some(reduce_get_validation_package),
        Action::HandleCustomSendResponse(_) => Some(reduce_handle_custom_send_response),
        Action::HandleQuery(_) => Some(reduce_handle_get_result),
        Action::HandleGetValidationPackage(_) => Some(reduce_handle_get_validation_package),
        Action::InitNetwork(_) => Some(reduce_init),
        Action::Publish(_) => Some(reduce_publish),
        Action::PublishHeaderEntry(_) => Some(reduce_publish_header_entry),
        Action::ResolveDirectConnection(_) => Some(reduce_resolve_direct_connection),
        Action::RespondAuthoringList(_) => Some(reduce_respond_authoring_list),
        Action::RespondGossipList(_) => Some(reduce_respond_gossip_list),
        Action::RespondFetch(_) => Some(reduce_respond_fetch_data),
        Action::RespondQuery(_) => Some(reduce_respond_query),
        Action::SendDirectMessage(_) => Some(reduce_send_direct_message),
        Action::SendDirectMessageTimeout(_) => Some(reduce_send_direct_message_timeout),
        Action::ShutdownNetwork => Some(reduce_shutdown),
        _ => None,
    }
}

pub fn reduce(
    old_state: Arc<NetworkState>,
    root_state: &State,
    action_wrapper: &ActionWrapper,
) -> Arc<NetworkState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NetworkState = (*old_state).clone();
            f(&mut new_state, &root_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

/// Sends the given Lib3hClientProtocol over the network using the network proxy instance
/// that lives in the NetworkState.
pub fn send(
    network_state: &mut NetworkState,
    msg: Lib3hClientProtocol,
) -> Result<(), HolochainError> {
    network_state
        .network
        .as_mut()
        .map(|mut network| {
            network.send(msg)
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .ok_or_else(|| HolochainError::ErrorGeneric("Network not initialized".to_string()))?
}

/// Sends the given DirectMessage to the node given by to_agent_id.
/// This creates a transient connection as every node-to-node communication follows a
/// request-response pattern. This function therefore logs the open connection
/// (expecting a response) in network_state.direct_message_connections.
pub fn send_message(
    network_state: &mut NetworkState,
    to_agent_id: &Address,
    message: DirectMessage,
) -> Result<(), HolochainError> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
    let id = format!("{}-{}", ProcessUniqueId::new().to_string(), rand_string);

    let content_json_string: JsonString = message.to_owned().into();
    let content = content_json_string.to_bytes();
    let space_address = network_state.dna_address.clone().unwrap();
    let data = DirectMessageData {
        request_id: id.clone(),
        space_address: space_address.into(),
        to_agent_id: to_agent_id.clone(),
        from_agent_id: network_state.agent_id.clone().unwrap().into(),
        content: content.into(),
    };

    let _ = send(network_state, Lib3hClientProtocol::SendDirectMessage(data))?;

    network_state.direct_message_connections.insert(id, message);

    Ok(())
}
