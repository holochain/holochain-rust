use holochain_core::{context::Context, persister::SimplePersister, signal::SignalSender};
use holochain_core_types::{
    agent::AgentId, eav::Attribute, error::HolochainError, sync::HcRwLock as RwLock,
};
use holochain_net::p2p_config::P2pConfig;
use holochain_persistence_api::{
    cas::storage::ContentAddressableStorage, eav::EntityAttributeValueStorage,
};
use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
use holochain_persistence_lmdb::{cas::lmdb::LmdbStorage, eav::lmdb::EavLmdbStorage};
use holochain_persistence_mem::{cas::memory::MemoryStorage, eav::memory::EavMemoryStorage};
use holochain_persistence_pickle::{cas::pickle::PickleStorage, eav::pickle::EavPickleStorage};

use jsonrpc_core::IoHandler;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    chain_storage: Option<Arc<RwLock<dyn ContentAddressableStorage>>>,
    dht_storage: Option<Arc<RwLock<dyn ContentAddressableStorage>>>,
    eav_storage: Option<Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>>,
    p2p_config: Option<P2pConfig>,
    conductor_api: Option<Arc<RwLock<IoHandler>>>,
    signal_tx: Option<SignalSender>,
    state_dump_logging: bool,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            instance_name: None,
            agent_id: None,
            chain_storage: None,
            dht_storage: None,
            eav_storage: None,
            p2p_config: None,
            conductor_api: None,
            signal_tx: None,
            state_dump_logging: false,
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
        let eav = //Arc<RwLock<holochain_persistence_api::eav::EntityAttributeValueStorage<Attribute>>> =
            Arc::new(RwLock::new(EavMemoryStorage::new()));
        self.chain_storage = Some(cas.clone());
        self.dht_storage = Some(cas);
        self.eav_storage = Some(eav);
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

        let file_storage = Arc::new(RwLock::new(FilesystemStorage::new(&cas_path)?));
        let eav_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>> =
            Arc::new(RwLock::new(EavFileStorage::new(eav_path)?));
        self.chain_storage = Some(file_storage.clone());
        self.dht_storage = Some(file_storage);
        self.eav_storage = Some(eav_storage);
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

        let file_storage = Arc::new(RwLock::new(PickleStorage::new(&cas_path)));
        let eav_storage = Arc::new(RwLock::new(EavPickleStorage::new(eav_path)));
        self.chain_storage = Some(file_storage.clone());
        self.dht_storage = Some(file_storage);
        self.eav_storage = Some(eav_storage);
        Ok(self)
    }

    /// Sets all three storages, chain, DHT and EAV storage, to persistent lmdb based implementations.
    /// Chain and DHT storages get set to the same pikcle CAS.
    /// Returns an error if no lmdb storage could be spawned on the given path.
    pub fn with_lmdb_storage<P: AsRef<Path>>(
        mut self,
        path: P,
        initial_mmap_bytes: Option<usize>,
    ) -> Result<Self, HolochainError> {
        let base_path: PathBuf = path.as_ref().into();
        let cas_path = base_path.join("cas");
        let eav_path = base_path.join("eav");
        fs::create_dir_all(&cas_path)?;
        fs::create_dir_all(&eav_path)?;

        let cas_storage = Arc::new(RwLock::new(LmdbStorage::new(&cas_path, initial_mmap_bytes)));
        let eav_storage = Arc::new(RwLock::new(EavLmdbStorage::new(
            eav_path,
            initial_mmap_bytes,
        )));
        self.chain_storage = Some(cas_storage.clone());
        self.dht_storage = Some(cas_storage);
        self.eav_storage = Some(eav_storage);
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

    /// Actually creates the context.
    /// Defaults to memory storages, an in-memory network config and a fake agent called "alice".
    /// The persister gets set to SimplePersister based on the chain storage.
    pub fn spawn(self) -> Context {
        let chain_storage = self
            .chain_storage
            .unwrap_or_else(|| Arc::new(RwLock::new(MemoryStorage::new())));
        let dht_storage = self
            .dht_storage
            .unwrap_or_else(|| Arc::new(RwLock::new(MemoryStorage::new())));
        let eav_storage = self
            .eav_storage
            .unwrap_or_else(|| Arc::new(RwLock::new(EavMemoryStorage::new())));

        Context::new(
            &self
                .instance_name
                .unwrap_or_else(|| "Anonymous-instance".to_string()),
            self.agent_id
                .unwrap_or_else(|| AgentId::generate_fake("alice")),
            Arc::new(RwLock::new(SimplePersister::new(chain_storage.clone()))),
            chain_storage,
            dht_storage,
            eav_storage,
            // TODO BLOCKER pass a peer list here?
            self.p2p_config
                .unwrap_or_else(P2pConfig::new_with_unique_memory_backend),
            self.conductor_api,
            self.signal_tx,
            self.state_dump_logging,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate tempfile;
    use self::tempfile::tempdir;
    use holochain_net::p2p_config::P2pBackendKind;
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
