use crate::{
    action::{Action, ActionWrapper},
    network::state::NetworkState,
    state::State,
};
use holochain_net::{
    connection::net_connection::NetSend, p2p_config::BackendConfig, p2p_network::P2pNetwork,
};
use holochain_persistence_api::cas::content::AddressableContent;
use lib3h_protocol::{data_types::SpaceData, protocol_client::Lib3hClientProtocol, Address};
use log::{debug, error, info};

pub fn reduce_init(state: &mut NetworkState, root_state: &State, action_wrapper: &ActionWrapper) {
    let action = action_wrapper.action();
    let network_settings = unwrap_to!(action => Action::InitNetwork);
    let handler = network_settings.handler.clone();
    let mut p2p_config = network_settings.p2p_config.clone();

    // Handle magic DNA property sim2h_url:
    // If our DNA sets a property with name "sim2h_url" and if this instance is configured
    // to use sim2h networking, override the conductor wide sim2h_url setting from the
    // conductor config with the DNA property's value.

    // Get the property from the DNA:
    let nucleus = root_state.nucleus();
    let dna = nucleus
        .dna
        .as_ref()
        .expect("No DNA found when initializing network!");
    let maybe_sim2h_url_override = dna
        .properties
        .as_object()
        .and_then(|props| props.get("sim2h_url"))
        .and_then(|sim2h_url_value| sim2h_url_value.as_str())
        .map(|sim2h_url_str| sim2h_url_str.to_string());

    // If we found a "sim2h_url" property...
    if let Some(sim2h_url) = maybe_sim2h_url_override {
        // ..and we're configured to use sim2h...
        if let BackendConfig::Sim2h(sim2h_config) = &mut p2p_config.backend_config {
            info!(
                "Found property 'sim2h_url' in DNA {} - overriding conductor wide sim2h URL with: {}",
                dna.address(),
                sim2h_url,
            );
            // ..override the conductor wide setting.
            sim2h_config.sim2h_url = sim2h_url;
        } else {
            debug!("DNA has 'sim2h_url' override property set, but it's ignored as we are not running a sim2h network backend");
        }
    }

    let mut network = P2pNetwork::new(
        handler,
        p2p_config,
        Some(Address::from(network_settings.agent_id.clone())),
        Some(root_state.conductor_api.clone()),
    )
    .unwrap();

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

    let json = Lib3hClientProtocol::JoinSpace(SpaceData {
        request_id: snowflake::ProcessUniqueId::new().to_string(),
        space_address: network_settings.dna_address.clone().into(),
        agent_id: network_settings.agent_id.clone().into(),
    });

    state.dna_address = Some(network_settings.dna_address.clone());
    state.agent_id = Some(network_settings.agent_id.clone());

    if let Err(err) = network.send(json) {
        error!("Could not send JsonProtocol::TrackDna. Error: {:?}", err);
        error!("Failed to initialize network!");
        network.stop();
        state.network = None;
    } else {
        state.network = Some(network);
    }
}

#[cfg(test)]
pub mod test {
    use self::tempfile::tempdir;
    use super::*;
    use crate::{
        context::Context,
        persister::SimplePersister,
        state::{test_store, StateWrapper},
    };
    use holochain_core_types::{agent::AgentId, dna::Dna};
    use holochain_locksmith::RwLock;
    use holochain_net::{connection::net_connection::NetHandler, p2p_config::P2pConfig};
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use holochain_tracing as ht;
    use std::sync::Arc;
    use tempfile;

    fn test_context() -> Arc<Context> {
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let mut context = Context::new(
            "Test-context-instance",
            AgentId::generate_fake("Terence"),
            Arc::new(RwLock::new(SimplePersister::new(file_storage.clone()))),
            file_storage.clone(),
            file_storage.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            P2pConfig::new_with_unique_memory_backend(),
            None,
            None,
            false,
            Arc::new(RwLock::new(
                holochain_metrics::DefaultMetricPublisher::default(),
            )),
            Arc::new(ht::null_tracer()),
        );

        let global_state = Arc::new(RwLock::new(StateWrapper::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    #[test]
    pub fn should_wait_for_protocol_p2p_ready() {
        let context: Arc<Context> = test_context();
        let dna_address: Address = context.agent_id.address();
        let agent_id = context.agent_id.content().to_string();
        let handler = NetHandler::new(Box::new(|_| Ok(())));
        let network_settings = crate::action::NetworkSettings {
            p2p_config: context.p2p_config.clone(),
            dna_address,
            agent_id,
            handler,
        };
        let action_wrapper = ActionWrapper::new(Action::InitNetwork(network_settings));

        let mut network_state = NetworkState::new();
        let mut root_state = test_store(context.clone());
        root_state = root_state.reduce(ActionWrapper::new(Action::InitializeChain(Dna::new())));

        let result = reduce_init(&mut network_state, &root_state, &action_wrapper);

        assert_eq!(result, ());
    }
}
