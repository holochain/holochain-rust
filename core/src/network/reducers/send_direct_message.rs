use crate::{
    action::{ActionWrapper, DirectMessageData},
    context::Context,
    network::{reducers::send, state::NetworkState},
};
use holochain_core_types::error::HolochainError;
use holochain_net_connection::protocol_wrapper::{MessageData, ProtocolWrapper};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    direct_message_data: &DirectMessageData,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let data = MessageData {
        msg_id: direct_message_data.msg_id.clone(),
        dna_address: network_state.dna_address.clone().unwrap(),
        to_agent_id: direct_message_data.address.to_string(),
        from_agent_id: network_state.agent_id.clone().unwrap(),
        data: serde_json::from_str(&serde_json::to_string(&direct_message_data.message).unwrap())
            .unwrap(),
    };

    let protocol_object = if direct_message_data.is_response {
        ProtocolWrapper::HandleSendResult(data)
    } else {
        network_state
            .direct_message_connections
            .insert(data.msg_id.clone(), direct_message_data.message.clone());
        ProtocolWrapper::SendMessage(data)
    };

    send(network_state, protocol_object)
}

pub fn reduce_send_direct_message(
    context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let dm_data = unwrap_to!(action => crate::action::Action::SendDirectMessage);
    if let Err(error) = inner(network_state, dm_data) {
        context.log(format!(
            "err/net: Error sending direct message: {:?}",
            error
        ));
    }
}

pub fn reduce_send_direct_message_timeout(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
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
        context::unique_mock_config,
        instance::tests::test_context,
        network::direct_message::{CustomDirectMessage, DirectMessage},
        state::test_store,
    };
    use holochain_core_types::{cas::content::Address, error::HolochainError};
    use std::sync::{Arc, RwLock};

    #[test]
    pub fn reduce_send_direct_message_timeout_test() {
        let mut context = test_context("alice");
        let store = test_store(context.clone());
        let store = Arc::new(RwLock::new(store));

        Arc::get_mut(&mut context).unwrap().set_state(store.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: unique_mock_config(),
            dna_address: "abcd".into(),
            agent_id: String::from("abcd"),
        }));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

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

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_reply = store
            .read()
            .unwrap()
            .network()
            .custom_direct_message_replys
            .get(&msg_id)
            .cloned();
        assert_eq!(maybe_reply, None);

        let action_wrapper = ActionWrapper::new(Action::SendDirectMessageTimeout(msg_id.clone()));
        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_reply = store
            .read()
            .unwrap()
            .network()
            .custom_direct_message_replys
            .get(&msg_id.clone())
            .cloned();
        assert_eq!(maybe_reply, Some(Err(HolochainError::Timeout)));
    }
}
