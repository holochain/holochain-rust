use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    network::{handler::create_handler, state::NetworkState},
};
use holochain_net::{
    connection::{
        json_protocol::{JsonProtocol, TrackDnaData},
        net_connection::NetSend,
        protocol::Protocol,
    },
    p2p_network::P2pNetwork,
};
use std::{
    sync::{mpsc::channel, Arc},
    time::Duration,
};

const P2P_READY_TIMEOUT_MS: u64 = 5000;

pub fn reduce_init(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let (sender, receiver) = channel();
    let mut network = P2pNetwork::new(
        create_handler(&context, network_settings.dna_address.to_string(), &sender),
        &network_settings.p2p_config,
    )
    .unwrap();

    let maybe_message = receiver.recv_timeout(Duration::from_millis(P2P_READY_TIMEOUT_MS));
    match maybe_message {
        Ok(Protocol::P2pReady) => context.log("debug/network/reducers: p2p networking ready"),
        Ok(message) => {
            context.log(format!(
                "warn/network/reducers: unexpected \
                 protocol message {:?}",
                message
            ));
        }
        Err(e) => {
            context.log(format!(
                "err/network/reducers: timed out waiting for p2p \
                 network to be ready: {:?}",
                e
            ));
            panic!(
                "p2p network not ready within alloted time of {:?} ms",
                P2P_READY_TIMEOUT_MS
            );
        }
    }

    // Configure network logger
    // Enable this for debugging network
    //    {
    //        let mut tweetlog = TWEETLOG.write().unwrap();
    //        tweetlog.set(LogLevel::Debug, None);
    //        // set level per tag
    //        tweetlog.set(LogLevel::Debug, Some("memory_server".to_string()));
    //        tweetlog.listen_to_tag("memory_server", Tweetlog::console);
    //        tweetlog.listen(Tweetlog::console);
    //        tweetlog.i("TWEETLOG ENABLED");
    //    }

    let json = JsonProtocol::TrackDna(TrackDnaData {
        dna_address: network_settings.dna_address.clone(),
        agent_id: network_settings.agent_id.clone(),
    });

    let _ = network.send(json.into()).and_then(|_| {
        state.network = Some(Arc::new(std::sync::Mutex::new(network)));
        state.dna_address = Some(network_settings.dna_address.clone());
        state.agent_id = Some(network_settings.agent_id.clone());
        Ok(())
    });
}

#[cfg(test)]
pub mod test {
    use self::tempfile::tempdir;
    use super::*;
    use crate::{logger::test_logger, persister::SimplePersister, state::State};
    use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use holochain_core_types::{agent::AgentId, cas::content::Address};
    use holochain_net::p2p_config::P2pConfig;
    use holochain_wasm_utils::holochain_core_types::cas::content::AddressableContent;
    use std::sync::{Mutex, RwLock};
    use tempfile;

    fn test_context() -> Arc<Context> {
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let mut context = Context::new(
            AgentId::generate_fake("Terence"),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
            file_storage.clone(),
            file_storage.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            P2pConfig::new_with_unique_memory_backend(),
            None,
            None,
        );

        let global_state = Arc::new(RwLock::new(State::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    #[test]
    pub fn should_wait_for_protocol_p2p_ready() {
        let context: Arc<Context> = test_context();
        let dna_address: Address = context.agent_id.address();
        let agent_id = context.agent_id.content().to_string();

        let network_settings = crate::action::NetworkSettings {
            p2p_config: context.p2p_config.clone(),
            dna_address,
            agent_id,
        };
        let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));

        let mut network_state = NetworkState::new();
        let result = reduce_init(context, &mut network_state, &action_wrapper);

        assert_eq!(result, ());
    }

}
