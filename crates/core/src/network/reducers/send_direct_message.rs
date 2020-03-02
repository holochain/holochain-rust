use crate::{
    action::{ActionWrapper, DirectMessageData},
    network::{reducers::send, state::NetworkState},
    state::State,
    
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use lib3h_protocol::{
    data_types::DirectMessageData as Lib3hDirectMessageData, protocol_client::Lib3hClientProtocol,
};

#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn inner(
    network_state: &mut NetworkState,
    direct_message_data: &DirectMessageData,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let content_json_string: JsonString = direct_message_data.message.to_owned().into();
    let content = content_json_string.to_bytes();
    let data = Lib3hDirectMessageData {
        request_id: direct_message_data.msg_id.clone(),
        space_address: network_state.dna_address.clone().unwrap().into(),
        to_agent_id: direct_message_data.address.clone().into(),
        from_agent_id: network_state.agent_id.clone().unwrap().into(),
        content: content.into(),
    };

    let protocol_object = if direct_message_data.is_response {
        Lib3hClientProtocol::HandleSendDirectMessageResult(data)
    } else {
        network_state
            .direct_message_connections
            .insert(data.request_id.clone(), direct_message_data.message.clone());
        Lib3hClientProtocol::SendDirectMessage(data)
    };

    send(network_state, protocol_object)
}

#[autotrace]
// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_send_direct_message(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (dm_data, maybe_timeout) = unwrap_to!(action => crate::action::Action::SendDirectMessage);
    if let Some(timeout) = maybe_timeout {
        network_state
            .direct_message_timeouts
            .insert(dm_data.msg_id.clone(), timeout.clone());
    }
    if let Err(error) = inner(network_state, dm_data) {
        println!("err/net: Error sending direct message: {:?}", error);
    }
}

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_send_direct_message_timeout(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => crate::action::Action::SendDirectMessageTimeout);

    network_state.direct_message_timeouts.remove(id);

    if network_state.custom_direct_message_replys.get(id).is_some() {
        return;
    }

    network_state
        .custom_direct_message_replys
        .insert(id.clone(), Err(HolochainError::Timeout));
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper, DirectMessageData, NetworkSettings},
        context::test_memory_network_config,
        instance::tests::test_context,
        network::{
            direct_message::{CustomDirectMessage, DirectMessage},
            handler::create_handler,
        },
        state::test_store,
    };
    use holochain_core_types::{dna::Dna, error::HolochainError};
    use holochain_persistence_api::cas::content::Address;

    #[test]
    pub fn reduce_send_direct_message_timeout_test() {
        let netname = Some("reduce_send_direct_message_timeout_test");
        let context = test_context("alice", netname);
        let mut store = test_store(context.clone());
        store = store.reduce(ActionWrapper::new(Action::InitializeChain(Dna::new())));

        let dna_address: Address = "reduce_send_direct_message_timeout_test".into();
        let handler = create_handler(&context, dna_address.to_string());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            p2p_config: test_memory_network_config(netname),
            dna_address: "reduce_send_direct_message_timeout_test".into(),
            agent_id: String::from("alice"),
            handler,
        }));

        store = store.reduce(action_wrapper);

        let custom_direct_message = DirectMessage::Custom(CustomDirectMessage {
            zome: String::from("test"),
            payload: Ok(String::from("test")),
        });
        let msg_id = String::from("any");
        let direct_message_data = DirectMessageData {
            address: Address::from("bogus"),
            message: custom_direct_message,
            msg_id: msg_id.clone(),
            is_response: false,
        };
        let action_wrapper =
            ActionWrapper::new(Action::SendDirectMessage((direct_message_data, None)));

        store = store.reduce(action_wrapper);

        let maybe_reply = store
            .network()
            .custom_direct_message_replys
            .get(&msg_id)
            .cloned();
        assert_eq!(maybe_reply, None);

        let action_wrapper = ActionWrapper::new(Action::SendDirectMessageTimeout(msg_id.clone()));
        store = store.reduce(action_wrapper);

        let maybe_reply = store
            .network()
            .custom_direct_message_replys
            .get(&msg_id.clone())
            .cloned();

        assert_eq!(maybe_reply, Some(Err(HolochainError::Timeout)));
    }
}
