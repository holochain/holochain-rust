use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::{file::EavFileStorage, memory::EavMemoryStorage},
    path::create_path_if_not_exists,
};
use holochain_core::{context::Context, logger::SimpleLogger, persister::SimplePersister};
use holochain_core_types::{
    agent::AgentId,
    cas::storage::ContentAddressableStorage,
    eav::EntityAttributeValueStorage,
    error::HolochainError,
    json::JsonString,
};
use holochain_net::p2p_config::P2pConfig;
use std::sync::{Arc, Mutex, RwLock};

pub struct ContextBuilder {
    agent_id: Option<AgentId>,
    //logger: Option<Arc<Mutex<Logger>>>,
    //persister: Option<Arc<Mutex<Persister>>>,
    chain_storage: Option<Arc<RwLock<ContentAddressableStorage>>>,
    dht_storage: Option<Arc<RwLock<ContentAddressableStorage>>>,
    eav_storage: Option<Arc<RwLock<EntityAttributeValueStorage>>>,
    network_config: Option<JsonString>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            agent_id: None,
            chain_storage: None,
            dht_storage: None,
            eav_storage: None,
            network_config: None,
        }
    }

    pub fn with_agent(&mut self, agent_id: AgentId) -> &mut Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_memory_storage(&mut self) -> &mut Self {
        let cas = Arc::new(RwLock::new(MemoryStorage::new()));
        let eav = Arc::new(RwLock::new(EavMemoryStorage::new()));
        self.chain_storage = Some(cas.clone());
        self.dht_storage = Some(cas);
        self.eav_storage = Some(eav);
        self
    }

    pub fn with_file_storage<T: Into<String>>(&mut self, path: T) -> Result<&mut Self, HolochainError> {
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

    pub fn with_network_config(&mut self, network_config: JsonString) -> &mut Self {
        self.network_config = Some(network_config);
        self
    }

    pub fn spawn(&self) -> Context {
        let chain_storage = self.chain_storage.clone().unwrap_or(Arc::new(RwLock::new(MemoryStorage::new())));
        let dht_storage = self.dht_storage.clone().unwrap_or(Arc::new(RwLock::new(MemoryStorage::new())));
        let eav_storage = self.eav_storage.clone().unwrap_or(Arc::new(RwLock::new(EavMemoryStorage::new())));
        Context::new(
            self.agent_id.clone().unwrap_or(AgentId::generate_fake("alice")),
            Arc::new(Mutex::new(SimpleLogger{})),
            Arc::new(Mutex::new(SimplePersister::new(chain_storage.clone()))),
            chain_storage,
            dht_storage,
            eav_storage,
            self.network_config.clone().unwrap_or(JsonString::from(String::from(P2pConfig::DEFAULT_MOCK_CONFIG))),
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
        assert_eq!(context.network_config, JsonString::from(String::from(P2pConfig::DEFAULT_MOCK_CONFIG)));
    }

    #[test]
    fn with_agent() {
        let agent = AgentId::generate_fake("alice");
        let context = ContextBuilder::new().with_agent(agent.clone()).spawn();
        assert_eq!(context.agent_id, agent);
    }

    #[test]
    fn with_network_config() {
        let net = JsonString::from(String::from(P2pConfig::DEFAULT_MOCK_CONFIG));
        let context = ContextBuilder::new().with_network_config(net.clone()).spawn();
        assert_eq!(context.network_config, net);
    }

    #[test]
    fn smoke_tests() {
        let _ = ContextBuilder::new().with_memory_storage().spawn();
        let temp = tempdir().expect("test was supposed to create temp dir");
        let temp_path = String::from(temp.path().to_str().expect("temp dir could not be string"));
        let _ = ContextBuilder::new().with_file_storage(temp_path).expect("Filestorage should get instantiated with tempdir").spawn();
    }
}