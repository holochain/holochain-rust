pub mod get_entry;
pub mod get_links;
pub mod get_validation_package;
pub mod handle_custom_send_response;
pub mod handle_get_links_result;
pub mod handle_get_result;
pub mod handle_get_validation_package;
pub mod init;
pub mod publish;
pub mod resolve_direct_connection;
pub mod respond_get;
pub mod respond_get_links;
pub mod send_direct_message;

use crate::{
    action::{Action, ActionWrapper, NetworkReduceFn},
    context::Context,
    network::{
        direct_message::DirectMessage,
        reducers::{
            get_entry::{reduce_get_entry, reduce_get_entry_timeout},
            get_links::{reduce_get_links, reduce_get_links_timeout},
            get_validation_package::reduce_get_validation_package,
            handle_custom_send_response::reduce_handle_custom_send_response,
            handle_get_result::reduce_handle_get_result,
            handle_get_links_result::reduce_handle_get_links_result,
            handle_get_validation_package::reduce_handle_get_validation_package,
            init::reduce_init,
            publish::reduce_publish,
            resolve_direct_connection::reduce_resolve_direct_connection,
            respond_get::reduce_respond_get,
            respond_get_links::reduce_respond_get_links,
            send_direct_message::{reduce_send_direct_message, reduce_send_direct_message_timeout},
        },
        state::NetworkState,
    },
};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{MessageData, ProtocolWrapper},
};
use snowflake::ProcessUniqueId;
use std::sync::Arc;

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<NetworkReduceFn> {
    match action_wrapper.action() {
        Action::GetEntry(_) => Some(reduce_get_entry),
        Action::GetEntryTimeout(_) => Some(reduce_get_entry_timeout),
        Action::GetLinks(_) => Some(reduce_get_links),
        Action::GetLinksTimeout(_) => Some(reduce_get_links_timeout),
        Action::GetValidationPackage(_) => Some(reduce_get_validation_package),
        Action::HandleCustomSendResponse(_) => Some(reduce_handle_custom_send_response),
        Action::HandleGetResult(_) => Some(reduce_handle_get_result),
        Action::HandleGetLinksResult(_) => Some(reduce_handle_get_links_result),
        Action::HandleGetValidationPackage(_) => Some(reduce_handle_get_validation_package),
        Action::InitNetwork(_) => Some(reduce_init),
        Action::Publish(_) => Some(reduce_publish),
        Action::ResolveDirectConnection(_) => Some(reduce_resolve_direct_connection),
        Action::RespondGet(_) => Some(reduce_respond_get),
        Action::RespondGetLinks(_) => Some(reduce_respond_get_links),
        Action::SendDirectMessage(_) => Some(reduce_send_direct_message),
        Action::SendDirectMessageTimeout(_) => Some(reduce_send_direct_message_timeout),
        _ => None,
    }
}

pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<NetworkState>,
    action_wrapper: &ActionWrapper,
) -> Arc<NetworkState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: NetworkState = (*old_state).clone();
            f(context, &mut new_state, &action_wrapper);
            Arc::new(new_state)
        }
        None => old_state,
    }
}

/// Sends the given ProtocolWrapper over the network using the network proxy instance
/// that lives in the NetworkState.
pub fn send(
    network_state: &mut NetworkState,
    protocol_wrapper: ProtocolWrapper,
) -> Result<(), HolochainError> {
    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(protocol_wrapper.into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .ok_or(HolochainError::ErrorGeneric(
            "Network not intialized".to_string(),
        ))?
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
    let id = ProcessUniqueId::new().to_string();

    let data = MessageData {
        msg_id: id.clone(),
        dna_address: network_state.dna_address.clone().unwrap(),
        to_agent_id: to_agent_id.to_string(),
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(&message).unwrap()).unwrap(),
    };

    let _ = send(network_state, ProtocolWrapper::SendMessage(data))?;

    network_state.direct_message_connections.insert(id, message);

    Ok(())
}
