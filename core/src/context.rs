use crate::{
    action::ActionWrapper,
    instance::Observer,
    logger::Logger,
    persister::Persister,
    signal::{Signal, SignalSender},
    state::State,
};
use holochain_core_types::{
    agent::AgentId,
    cas::storage::ContentAddressableStorage,
    dna::{wasm::DnaWasm, Dna},
    eav::EntityAttributeValueStorage,
    error::{HcResult, HolochainError},
    json::JsonString,
};
use holochain_net::p2p_config::P2pConfig;
use jsonrpc_ws_server::jsonrpc_core::IoHandler;
use std::{
    sync::{mpsc::SyncSender, Arc, Mutex, RwLock, RwLockReadGuard},
    thread::sleep,
    time::Duration,
};

#[derive(Clone)]
pub struct ContextStateful {
    state: Arc<RwLock<State>>,
    ctx: Arc<ContextOnly>,
}

impl ContextStateful {
    pub fn new(ctx: Arc<ContextOnly>, state: Arc<RwLock<State>>) -> Self {
        Self { ctx, state }
    }
    pub fn agent_id(&self) -> &AgentId {
        &self.ctx.agent_id
    }
    pub fn logger(&self) -> Arc<Mutex<Logger>> {
        self.ctx.logger.clone()
    }
    pub fn persister(&self) -> Arc<Mutex<Persister>> {
        self.ctx.persister.clone()
    }
    pub fn action_channel(&self) -> &SyncSender<ActionWrapper> {
        self.ctx.action_channel()
    }
    pub fn observer_channel(&self) -> &SyncSender<Observer> {
        self.ctx.observer_channel()
    }
    pub fn chain_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.ctx.chain_storage.clone()
    }
    pub fn dht_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.ctx.dht_storage.clone()
    }
    pub fn eav_storage(&self) -> Arc<RwLock<EntityAttributeValueStorage>> {
        self.ctx.eav_storage.clone()
    }
    pub fn network_config(&self) -> &JsonString {
        &self.ctx.network_config
    }
    pub fn container_api(&self) -> Option<&Arc<RwLock<IoHandler>>> {
        self.ctx.container_api.as_ref()
    }
    pub fn signal_tx(&self) -> Option<&SyncSender<Signal>> {
        self.ctx.signal_tx.as_ref()
    }
    pub fn state(&self) -> RwLockReadGuard<State> {
        self.state.read().unwrap()
    }

    pub fn log<T: Into<String>>(&self, msg: T) {
        self.log(msg)
    }

    pub fn context_only(&self) -> Arc<ContextOnly> {
        self.ctx.clone()
    }

    pub fn get_dna(&self) -> Option<Dna> {
        use std::{thread, time::Duration};
        // In the case of genesis we encounter race conditions with regards to setting the DNA.
        // Genesis gets called asynchronously right after dispatching an action that sets the DNA in
        // the state, which can result in this code being executed first.
        // But we can't run anything if there is no DNA which holds the WASM, so we have to wait here.
        // TODO: use a future here
        let mut dna = None;
        let mut done = false;
        let mut tries = 0;
        while !done {
            {
                dna = self.state.read().unwrap().nucleus().dna();
            }
            match dna {
                Some(_) => done = true,
                None => {
                    if tries > 10 {
                        done = true;
                    } else {
                        thread::sleep(Duration::from_millis(10));
                        tries += 1;
                    }
                }
            }
        }
        dna
    }

    pub fn get_wasm(&self, zome: &str) -> Option<DnaWasm> {
        let dna = self.get_dna().expect("Callback called without DNA set!");
        dna.get_wasm_from_zome_name(zome)
            .and_then(|wasm| Some(wasm.clone()).filter(|_| !wasm.code.is_empty()))
    }
}

/// ContextOnly holds the components that parts of a Holochain instance need in order to operate.
/// This includes components that are injected from the outside like logger and persister
/// but also the store of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct ContextOnly {
    pub agent_id: AgentId,
    pub logger: Arc<Mutex<Logger>>,
    pub persister: Arc<Mutex<Persister>>,
    pub action_channel: Option<SyncSender<ActionWrapper>>,
    pub observer_channel: Option<SyncSender<Observer>>,
    pub chain_storage: Arc<RwLock<ContentAddressableStorage>>,
    pub dht_storage: Arc<RwLock<ContentAddressableStorage>>,
    pub eav_storage: Arc<RwLock<EntityAttributeValueStorage>>,
    pub network_config: JsonString,
    pub container_api: Option<Arc<RwLock<IoHandler>>>,
    pub signal_tx: Option<SyncSender<Signal>>,
}

impl ContextOnly {
    pub fn default_channel_buffer_size() -> usize {
        100
    }

    pub fn new(
        agent_id: AgentId,
        logger: Arc<Mutex<Logger>>,
        persister: Arc<Mutex<Persister>>,
        chain_storage: Arc<RwLock<ContentAddressableStorage>>,
        dht_storage: Arc<RwLock<ContentAddressableStorage>>,
        eav: Arc<RwLock<EntityAttributeValueStorage>>,
        network_config: JsonString,
        container_api: Option<Arc<RwLock<IoHandler>>>,
        signal_tx: Option<SignalSender>,
    ) -> Self {
        ContextOnly {
            agent_id,
            logger,
            persister,
            action_channel: None,
            signal_tx: signal_tx,
            observer_channel: None,
            chain_storage,
            dht_storage,
            eav_storage: eav,
            network_config,
            container_api,
        }
    }

    pub fn new_with_channels(
        agent_id: AgentId,
        logger: Arc<Mutex<Logger>>,
        persister: Arc<Mutex<Persister>>,
        action_channel: Option<SyncSender<ActionWrapper>>,
        signal_tx: Option<SyncSender<Signal>>,
        observer_channel: Option<SyncSender<Observer>>,
        cas: Arc<RwLock<ContentAddressableStorage>>,
        eav: Arc<RwLock<EntityAttributeValueStorage>>,
        network_config: JsonString,
    ) -> Result<ContextOnly, HolochainError> {
        Ok(ContextOnly {
            agent_id,
            logger,
            persister,
            action_channel,
            signal_tx,
            observer_channel,
            chain_storage: cas.clone(),
            dht_storage: cas,
            eav_storage: eav,
            network_config,
            container_api: None,
        })
    }

    // helper function to make it easier to call the logger
    pub fn log<T: Into<String>>(&self, msg: T) {
        let mut logger = self
            .logger
            .lock()
            .or(Err(HolochainError::LoggingError))
            .expect("Logger should work");;
        logger.log(msg.into());
    }

    // @NB: these three getters smell bad because previously Instance and ContextOnly had SyncSenders
    // rather than Option<SyncSenders>, but these would be initialized by default to broken channels
    // which would panic if `send` was called upon them. These `expect`s just bring more visibility to
    // that potential failure mode.
    // @see https://github.com/holochain/holochain-rust/issues/739
    pub fn action_channel(&self) -> &SyncSender<ActionWrapper> {
        self.action_channel
            .as_ref()
            .expect("Action channel not initialized")
    }

    pub fn signal_tx(&self) -> Option<&SyncSender<Signal>> {
        self.signal_tx.as_ref()
    }

    pub fn observer_channel(&self) -> &SyncSender<Observer> {
        self.observer_channel
            .as_ref()
            .expect("Observer channel not initialized")
    }

    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    pub fn logger(&self) -> Arc<Mutex<Logger>> {
        self.logger.clone()
    }
    pub fn persister(&self) -> Arc<Mutex<Persister>> {
        self.persister.clone()
    }
    pub fn chain_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.chain_storage.clone()
    }
    pub fn dht_storage(&self) -> Arc<RwLock<ContentAddressableStorage>> {
        self.dht_storage.clone()
    }
    pub fn eav_storage(&self) -> Arc<RwLock<EntityAttributeValueStorage>> {
        self.eav_storage.clone()
    }
    pub fn network_config(&self) -> &JsonString {
        &self.network_config
    }
    pub fn container_api(&self) -> Option<&Arc<RwLock<IoHandler>>> {
        self.container_api.as_ref()
    }
}

pub async fn get_dna_and_agent(context: &Arc<ContextStateful>) -> HcResult<(String, String)> {
    let state = context.state();
    let agent_state = state.agent();

    let agent = await!(agent_state.get_agent(&context))?;
    let agent_id = agent.key;

    let dna = state
        .nucleus()
        .dna()
        .ok_or("Network::start() called without DNA".to_string())?;
    let dna_hash = base64::encode(&dna.multihash()?);
    Ok((dna_hash, agent_id))
}

/// create a test network
#[cfg_attr(tarpaulin, skip)]
pub fn mock_network_config() -> JsonString {
    JsonString::from(P2pConfig::DEFAULT_MOCK_CONFIG)
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    extern crate test_utils;
    use self::tempfile::tempdir;
    use super::*;
    use crate::{
        context::mock_network_config, instance::tests::test_logger, persister::SimplePersister,
        state::State,
    };
    use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use holochain_core_types::agent::AgentId;
    use std::sync::{Arc, Mutex, RwLock};

    #[test]
    fn default_buffer_size_test() {
        assert_eq!(ContextOnly::default_channel_buffer_size(), 100);
    }
}
