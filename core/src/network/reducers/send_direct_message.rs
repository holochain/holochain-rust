use crate::{
    action::{ActionWrapper, DirectMessageData},
    network::{reducers::send, state::NetworkState},
    state::State,
};
use holochain_core_types::{error::HolochainError, json::JsonString};
use holochain_net::connection::json_protocol::{JsonProtocol, MessageData};

fn inner(
    network_state: &mut NetworkState,
    direct_message_data: &DirectMessageData,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let content_json_string: JsonString = direct_message_data.message.to_owned().into();
    let content = content_json_string.to_bytes();
    let data = MessageData {
        request_id: direct_message_data.msg_id.clone(),
        dna_address: network_state.dna_address.clone().unwrap(),
        to_agent_id: direct_message_data.address.clone(),
        from_agent_id: network_state.agent_id.clone().unwrap().into(),
        content,
    };

    let protocol_object = if direct_message_data.is_response {
        JsonProtocol::HandleSendMessageResult(data)
    } else {
        network_state
            .direct_message_connections
            .insert(data.request_id.clone(), direct_message_data.message.clone());
        JsonProtocol::SendMessage(data)
    };

    send(network_state, protocol_object)
}

pub fn reduce_send_direct_message(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let dm_data = unwrap_to!(action => crate::action::Action::SendDirectMessage);
    if let Err(error) = inner(network_state, dm_data) {
        println!("err/net: Error sending direct message: {:?}", error);
    }
}

pub fn reduce_send_direct_message_timeout(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let id = unwrap_to!(action => crate::action::Action::SendDirectMessageTimeout);

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
    use holochain_core_types::{cas::content::Address, error::HolochainError};

    #[test]
    pub fn reduce_send_direct_message_timeout_test() {
        let netname = Some("reduce_send_direct_message_timeout_test");
        let context = test_context("alice", netname);
        let mut store = test_store(context.clone());

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
        let action_wrapper = ActionWrapper::new(Action::SendDirectMessage(direct_message_data));

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
