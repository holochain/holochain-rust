use holochain_core::{context::Context, persister::SimplePersister, signal::SignalSender};
use holochain_core_types::{agent::AgentId, eav::Attribute, error::HolochainError};
use holochain_locksmith::RwLock;
use holochain_net::p2p_config::P2pConfig;
use holochain_persistence_api::txn::PersistenceManagerDyn;

use jsonrpc_core::IoHandler;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use holochain_metrics::{DefaultMetricPublisher, MetricPublisher, MetricPublisherConfig};

/// This type helps building [context objects](struct.Context.html) that need to be
/// passed in to Holochain intances.
///
/// This is typically needed in any conductor implementation but also in almost every test.
/// Follows the [builder pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
///
/// Use any combination of `with_*` functions to configure the context and finally call
/// `spawn()` to retrieve the context.
pub struct ContextBuilder {
    instance_name: Option<String>,
    agent_id: Option<AgentId>,
    // Persister is currently set to a reasonable default in spawn().
    // TODO: add with_persister() function to ContextBuilder.
    //persister: Option<Arc<Mutex<Persister>>>,
    persistence_manager: Option<Arc<dyn PersistenceManagerDyn<Attribute>>>,
    p2p_config: Option<P2pConfig>,
    conductor_api: Option<Arc<RwLock<IoHandler>>>,
    signal_tx: Option<SignalSender>,
    state_dump_logging: bool,
    metric_publisher: Option<Arc<RwLock<dyn MetricPublisher>>>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            instance_name: None,
            agent_id: None,
            persistence_manager: None,
            p2p_config: None,
            conductor_api: None,
            signal_tx: None,
            state_dump_logging: false,
            metric_publisher: None,
        }
    }

    /// Sets the agent of the context that gets built.
    pub fn with_agent(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Sets all three storages, chain, DHT and EAV storage, to transient memory implementations.
    /// Chain and DHT storages get set to the same memory CAS.
    pub fn with_memory_storage(mut self) -> Self {
        let persistence_manager = Arc::new(holochain_persistence_mem::txn::new_manager());
        self.persistence_manager = Some(persistence_manager);
        self
    }

    /// Sets all three storages, chain, DHT and EAV storage, to persistent file based implementations.
    /// Chain and DHT storages get set to the same file CAS.
    /// Returns an error if no file storage could be spawned on the given path.
    pub fn with_file_storage<P: AsRef<Path>>(mut self, path: P) -> Result<Self, HolochainError> {
        let base_path: PathBuf = path.as_ref().into();
        let cas_path = base_path.join("cas");
        let eav_path = base_path.join("eav");
        fs::create_dir_all(&cas_path)?;
        fs::create_dir_all(&eav_path)?;

        let persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>> =
            Arc::new(holochain_persistence_file::txn::new_manager(cas_path, eav_path).unwrap());

        self.persistence_manager = Some(persistence_manager);
        Ok(self)
    }

    /// Sets all three storages, chain, DHT and EAV storage, to persistent pikcle based implementations.
    /// Chain and DHT storages get set to the same pikcle CAS.
    /// Returns an error if no pickle storage could be spawned on the given path.
    pub fn with_pickle_storage<P: AsRef<Path>>(mut self, path: P) -> Result<Self, HolochainError> {
        let base_path: PathBuf = path.as_ref().into();
        let cas_path = base_path.join("cas");
        let eav_path = base_path.join("eav");
        fs::create_dir_all(&cas_path)?;
        fs::create_dir_all(&eav_path)?;
        let persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>> = Arc::new(
            holochain_persistence_pickle::txn::new_manager(cas_path, eav_path),
        );
        self.persistence_manager = Some(persistence_manager);
        Ok(self)
    }

    /// Sets all three storages, chain, DHT and EAV storage, to persistent lmdb based implementations.
    /// Chain and DHT storages get set to the same pikcle CAS.
    /// Returns an error if no lmdb storage could be spawned on the given path.
    pub fn with_lmdb_storage<P: AsRef<Path>, P2: AsRef<Path> + Clone>(
        mut self,
        path: P,
        staging_path_prefix: Option<P2>,
        initial_mmap_bytes: Option<usize>,
    ) -> Result<Self, HolochainError> {
        let env_path: PathBuf = path.as_ref().into();

        let persistence_manager: Arc<dyn PersistenceManagerDyn<Attribute>> =
            Arc::new(holochain_persistence_lmdb::txn::new_manager(
                env_path,
                staging_path_prefix,
                initial_mmap_bytes,
                None,
                None,
                None,
            ));
        self.persistence_manager = Some(persistence_manager);
        Ok(self)
    }

    /// Sets the network config.
    pub fn with_p2p_config(mut self, p2p_config: P2pConfig) -> Self {
        self.p2p_config = Some(p2p_config);
        self
    }

    pub fn with_conductor_api(mut self, api_handler: IoHandler) -> Self {
        self.conductor_api = Some(Arc::new(RwLock::new(api_handler)));
        self
    }

    pub fn with_signals(mut self, signal_tx: SignalSender) -> Self {
        self.signal_tx = Some(signal_tx);
        self
    }

    pub fn with_instance_name(mut self, instance_name: &str) -> Self {
        self.instance_name = Some(String::from(instance_name));
        self
    }

    pub fn with_state_dump_logging(mut self) -> Self {
        self.state_dump_logging = true;
        self
    }

    pub fn with_metric_publisher(mut self, config: &MetricPublisherConfig) -> Self {
        let config = match &config {
            MetricPublisherConfig::CloudWatchLogs(config) => {
                let log_stream_name = config.clone().log_stream_name.map(|log_stream_name| {
                    self.instance_name
                        .clone()
                        .map(|instance_name| format!("{}.{}", log_stream_name, instance_name))
                        .unwrap_or_else(|| log_stream_name)
                });
                MetricPublisherConfig::CloudWatchLogs(
                    holochain_metrics::config::CloudWatchLogsConfig {
                        log_stream_name,
                        ..config.clone()
                    },
                )
            }
            MetricPublisherConfig::Logger => MetricPublisherConfig::Logger,
        };
        self.metric_publisher = Some(config.create_metric_publisher());
        self
    }

    /// Actually creates the context.
    /// Defaults to memory storages, an in-memory network config and a fake agent called "alice".
    /// The persister gets set to SimplePersister based on the chain storage.
    pub fn spawn(self) -> Context {
        let persistence_manager = self
            .persistence_manager
            .unwrap_or_else(|| Arc::new(holochain_persistence_mem::txn::new_manager()));
        let metric_publisher = self
            .metric_publisher
            .unwrap_or_else(|| Arc::new(RwLock::new(DefaultMetricPublisher::default())));

        Context::new(
            &self
                .instance_name
                .unwrap_or_else(|| "Anonymous-instance".to_string()),
            self.agent_id
                .unwrap_or_else(|| AgentId::generate_fake("alice")),
            Arc::new(RwLock::new(SimplePersister::new(
                persistence_manager.clone(),
            ))),
            persistence_manager,
            // TODO BLOCKER pass a peer list here?
            self.p2p_config
                .unwrap_or_else(P2pConfig::new_with_unique_memory_backend),
            self.conductor_api,
            self.signal_tx,
            self.state_dump_logging,
            metric_publisher,
        )
    }
}

#[cfg(test)]
mod tests {
    use self::tempfile::tempdir;
    use super::*;
    use holochain_net::p2p_config::P2pBackendKind;
    use tempfile;
    use test_utils::mock_signing::mock_conductor_api;

    #[test]
    fn vanilla() {
        let agent = AgentId::generate_fake("alice");
        let context = ContextBuilder::new()
            .with_conductor_api(mock_conductor_api(agent.clone()))
            .spawn();
        assert_eq!(context.agent_id, agent);
        assert_eq!(
            P2pBackendKind::LegacyInMemory,
            context.p2p_config.backend_kind
        );
    }

    #[test]
    fn with_agent() {
        let agent = AgentId::generate_fake("alice");
        let context = ContextBuilder::new()
            .with_agent(agent.clone())
            .with_conductor_api(mock_conductor_api(agent.clone()))
            .spawn();
        assert_eq!(context.agent_id, agent);
    }

    #[test]
    fn with_network_config() {
        let net = P2pConfig::new_with_unique_memory_backend();
        let context = ContextBuilder::new()
            .with_p2p_config(net.clone())
            .with_conductor_api(mock_conductor_api(AgentId::generate_fake("alice")))
            .spawn();
        assert_eq!(context.p2p_config, net);
    }

    #[test]
    fn smoke_tests() {
        let _ = ContextBuilder::new()
            .with_memory_storage()
            .with_conductor_api(mock_conductor_api(AgentId::generate_fake("alice")))
            .spawn();
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let _ = ContextBuilder::new()
            .with_file_storage(temp_path)
            .expect("Filestorage should get instantiated with tempdir")
            .with_conductor_api(mock_conductor_api(AgentId::generate_fake("alice")))
            .spawn();
    }
}
