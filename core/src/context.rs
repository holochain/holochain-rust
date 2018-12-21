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

/// Context holds the components that parts of a Holochain instance need in order to operate.
/// This includes components that are injected from the outside like logger and persister
/// but also the store of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct Context {
    pub agent_id: AgentId,
    pub logger: Arc<Mutex<Logger>>,
    pub persister: Arc<Mutex<Persister>>,
    state: Option<Arc<RwLock<State>>>,
    pub action_channel: Option<SyncSender<ActionWrapper>>,
    pub observer_channel: Option<SyncSender<Observer>>,
    pub chain_storage: Arc<RwLock<ContentAddressableStorage>>,
    pub dht_storage: Arc<RwLock<ContentAddressableStorage>>,
    pub eav_storage: Arc<RwLock<EntityAttributeValueStorage>>,
    pub network_config: JsonString,
    pub container_api: Option<Arc<RwLock<IoHandler>>>,
    pub signal_tx: Option<SyncSender<Signal>>,
}

impl Context {
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
        Context {
            agent_id,
            logger,
            persister,
            state: None,
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
    ) -> Result<Context, HolochainError> {
        Ok(Context {
            agent_id,
            logger,
            persister,
            state: None,
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

    pub fn set_state(&mut self, state: Arc<RwLock<State>>) {
        self.state = Some(state);
    }

    pub fn state(&self) -> Option<RwLockReadGuard<State>> {
        match self.state {
            None => None,
            Some(ref s) => Some(s.read().unwrap()),
        }
    }

    pub fn get_dna(&self) -> Option<Dna> {
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
                let state = self
                    .state()
                    .expect("Callback called without application state!");
                dna = state.nucleus().dna();
            }
            match dna {
                Some(_) => done = true,
                None => {
                    if tries > 10 {
                        done = true;
                    } else {
                        sleep(Duration::from_millis(10));
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

    // @NB: these three getters smell bad because previously Instance and Context had SyncSenders
    // rather than Option<SyncSenders>, but these would be initialized by default to broken channels
    // which would panic if `send` was called upon them. These `expect`s just bring more visibility to
    // that potential failure mode.
    // @see https://github.com/holochain/holochain-rust/issues/739
    pub fn action_channel(&self) -> &SyncSender<ActionWrapper> {
        self.action_channel
            .as_ref()
            .expect("Action channel not initialized")
    }

    pub fn signal_tx(&self) -> &SyncSender<Signal> {
        self.signal_tx
            .as_ref()
            .expect("Signal channel not initialized")
    }

    pub fn observer_channel(&self) -> &SyncSender<Observer> {
        self.observer_channel
            .as_ref()
            .expect("Observer channel not initialized")
    }
}

pub async fn get_dna_and_agent(context: &Arc<Context>) -> HcResult<(String, String)> {
    let state = context
        .state()
        .ok_or("Network::start() could not get application state".to_string())?;
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
        assert_eq!(Context::default_channel_buffer_size(), 100);
    }

    #[test]
    fn state_test() {
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let mut maybe_context = Context::new(
            AgentId::generate_fake("Terence"),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
            file_storage.clone(),
            file_storage.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            mock_network_config(),
            None,
            None,
        );

        assert!(maybe_context.state().is_none());

        let global_state = Arc::new(RwLock::new(State::new(Arc::new(maybe_context.clone()))));
        maybe_context.set_state(global_state.clone());

        {
            let _read_lock = global_state.read().unwrap();
            assert!(maybe_context.state().is_some());
        }
    }

    #[test]
    #[should_panic]
    #[cfg(not(windows))] // RwLock does not panic on windows since mutexes are recursive
    fn test_deadlock() {
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
            mock_network_config(),
            None,
            None,
        );

        let global_state = Arc::new(RwLock::new(State::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());

        {
            let _write_lock = global_state.write().unwrap();
            // This line panics because we would enter into a deadlock
            context.state();
        }
    }
}
