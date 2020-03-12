use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn check_network_processes_for_timeouts(context: Arc<Context>) {
    let state = context.state().expect("Couldn't get state in timeout job");
    for (key, (time, duration)) in state.network().query_timeouts.iter() {
        if let Ok(elapsed) = time.elapsed() {
            if elapsed > *duration {
                dispatch_action(
                    context.action_channel(),
                    ActionWrapper::new(Action::QueryTimeout(key.clone())),
                );
            }
        }
    }

    for (key, (time, duration)) in state.network().direct_message_timeouts.iter() {
        if let Ok(elapsed) = time.elapsed() {
            if elapsed > *duration {
                dispatch_action(
                    context.action_channel(),
                    ActionWrapper::new(Action::SendDirectMessageTimeout(key.clone())),
                );
            }
        }
    }

    for (address, (time, duration)) in state.network().get_validation_package_timeouts.iter() {
        if let Ok(elapsed) = time.elapsed() {
            if elapsed > *duration {
                dispatch_action(
                    context.action_channel(),
                    ActionWrapper::new(Action::GetValidationPackageTimeout(address.clone())),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        action::{Action, ActionWrapper, DirectMessageData},
        instance::{dispatch_action, tests::test_context, Instance},
        network::direct_message::{CustomDirectMessage, DirectMessage},
    };
    use bitflags::_core::time::Duration;
    use holochain_core_types::error::HolochainError;
    use holochain_persistence_api::cas::content::Address;
    use std::time::SystemTime;

    #[test]
    pub fn reduce_send_direct_message_timeout_test() {
        let netname = Some("can_commit_dna");
        let context = test_context("alex", netname);
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);
        let mut instance = Instance::new(context.clone());
        let context = instance
            .initialize(Some(dna.clone()), context.clone())
            .unwrap();

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
        let action_wrapper = ActionWrapper::new(Action::SendDirectMessage((
            direct_message_data,
            Some((SystemTime::now(), Duration::from_millis(500))),
        )));

        dispatch_action(context.action_channel(), action_wrapper);

        std::thread::sleep(Duration::from_secs(1));

        let maybe_reply = context
            .state()
            .unwrap()
            .network()
            .custom_direct_message_replys
            .get(&msg_id.clone())
            .cloned();

        assert_eq!(
            maybe_reply,
            Some(Err(HolochainError::Timeout(
                "timeout src: crates/core/src/network/reducers/send_direct_message.rs:158"
                    .to_string()
            )))
        );
    }
}
