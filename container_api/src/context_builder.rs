use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::{file::EavFileStorage, memory::EavMemoryStorage},
    path::create_path_if_not_exists,
};

use holochain_core::{
    context::Context,
    logger::{Logger, SimpleLogger},
    persister::SimplePersister,
    signal::SignalSender,
};
use holochain_core_types::{
    agent::AgentId, cas::storage::ContentAddressableStorage, eav::EntityAttributeValueStorage,
    error::HolochainError, json::JsonString,
};
use holochain_net::p2p_config::P2pConfig;
use jsonrpc_ws_server::jsonrpc_core::IoHandler;
use std::sync::{Arc, Mutex, RwLock};

/// This type helps building [context objects](struct.Context.html) that need to be
/// passed in to Holochain intances.
///
/// This is typically needed in any container implementation but also in almost every test.
/// Follows the [builder pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
///
/// Use any combination of `with_*` functions to configure the context and finally call
/// `spawn()` to retrieve the context.
pub struct ContextBuilder {
    agent_id: Option<AgentId>,
    logger: Option<Arc<Mutex<Logger>>>,
    // Persister is currently set to a reasonable default in spawn().
    // TODO: add with_persister() function to ContextBuilder.
    //persister: Option<Arc<Mutex<Persister>>>,
    chain_storage: Option<Arc<RwLock<ContentAddressableStorage>>>,
    dht_storage: Option<Arc<RwLock<ContentAddressableStorage>>>,
    eav_storage: Option<Arc<RwLock<EntityAttributeValueStorage>>>,
    network_config: Option<JsonString>,
    container_api: Option<Arc<RwLock<IoHandler>>>,
    signal_tx: Option<SignalSender>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            agent_id: None,
            logger: None,
            chain_storage: None,
            dht_storage: None,
            eav_storage: None,
            network_config: None,
            container_api: None,
            signal_tx: None,
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
        let cas = Arc::new(RwLock::new(MemoryStorage::new()));
        let eav = Arc::new(RwLock::new(EavMemoryStorage::new()));
        self.chain_storage = Some(cas.clone());
        self.dht_storage = Some(cas);
        self.eav_storage = Some(eav);
        self
    }

    /// Sets all three storages, chain, DHT and EAV storage, to persistent file based implementations.
    /// Chain and DHT storages get set to the same file CAS.
    /// Returns an error if no file storage could be spawned on the given path.
    pub fn with_file_storage<T: Into<String>>(mut self, path: T) -> Result<Self, HolochainError> {
        let path: String = path.into();
        let cas_path = format!("{}/cas", path);
        let eav_path = format!("{}/eav", path);
        create_path_if_not_exists(&cas_path)?;
        create_path_if_not_exists(&eav_path)?;

        let file_storage = Arc::new(RwLock::new(FilesystemStorage::new(&cas_path)?));
        let eav_storage = Arc::new(RwLock::new(EavFileStorage::new(eav_path)?));
        self.chain_storage = Some(file_storage.clone());
        self.dht_storage = Some(file_storage);
        self.eav_storage = Some(eav_storage);
        Ok(self)
    }

    /// Sets the network config.
    pub fn with_network_config(mut self, network_config: JsonString) -> Self {
        self.network_config = Some(network_config);
        self
    }

    pub fn with_container_api(mut self, api_handler: IoHandler) -> Self {
        self.container_api = Some(Arc::new(RwLock::new(api_handler)));
        self
    }

    pub fn with_logger(mut self, logger: Arc<Mutex<Logger>>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn with_signals(mut self, signal_tx: SignalSender) -> Self {
        self.signal_tx = Some(signal_tx);
        self
    }

    /// Actually creates the context.
    /// Defaults to memory storages, an in-memory network config and a fake agent called "alice".
    /// The logger gets set to SimpleLogger.
    /// The persister gets set to SimplePersister based on the chain storage.
    pub fn spawn(self) -> Context {
        let chain_storage = self
            .chain_storage
            .unwrap_or(Arc::new(RwLock::new(MemoryStorage::new())));
        let dht_storage = self
            .dht_storage
            .unwrap_or(Arc::new(RwLock::new(MemoryStorage::new())));
        let eav_storage = self
            .eav_storage
            .unwrap_or(Arc::new(RwLock::new(EavMemoryStorage::new())));
        Context::new(
            self.agent_id.unwrap_or(AgentId::generate_fake("alice")),
            self.logger.unwrap_or(Arc::new(Mutex::new(SimpleLogger {}))),
            Arc::new(Mutex::new(SimplePersister::new(chain_storage.clone()))),
            chain_storage,
            dht_storage,
            eav_storage,
            self.network_config.unwrap_or(JsonString::from(String::from(
                P2pConfig::unique_memory_backend_string(),
            ))),
            self.container_api,
            self.signal_tx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn vanilla() {
        let context = ContextBuilder::new().spawn();
        assert_eq!(context.agent_id, AgentId::generate_fake("alice"));
        assert!(context
            .network_config
            .to_string()
            .contains(r#""backend_kind": "MEMORY""#));
    }

    #[test]
    fn with_agent() {
        let agent = AgentId::generate_fake("alice");
        let context = ContextBuilder::new().with_agent(agent.clone()).spawn();
        assert_eq!(context.agent_id, agent);
    }

    #[test]
    fn with_network_config() {
        let net = JsonString::from(String::from(P2pConfig::unique_memory_backend_string()));
        let context = ContextBuilder::new()
            .with_network_config(net.clone())
            .spawn();
        assert_eq!(context.network_config, net);
    }

    #[test]
    fn smoke_tests() {
        let _ = ContextBuilder::new().with_memory_storage().spawn();
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let _ = ContextBuilder::new()
            .with_file_storage(temp_path)
            .expect("Filestorage should get instantiated with tempdir")
            .spawn();
    }
}
