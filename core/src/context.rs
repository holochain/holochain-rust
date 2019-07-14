use crate::{
    action::{Action, ActionWrapper},
    conductor_api::ConductorApi,
    instance::Observer,
    logger::Logger,
    nucleus::actions::get_entry::get_entry_from_cas,
    persister::Persister,
    signal::{Signal, SignalSender},
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::{
    task::{noop_waker_ref, Poll},
    Future,
};

use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::EntityAttributeValueStorage,
};

use crate::state::StateWrapper;
use holochain_core_types::{
    agent::AgentId,
    dna::{wasm::DnaWasm, Dna},
    eav::Attribute,
    entry::{
        cap_entries::{CapabilityType, ReservedCapabilityId},
        entry_type::EntryType,
        Entry,
    },
    error::{HcResult, HolochainError},
};
use holochain_net::p2p_config::P2pConfig;
use jsonrpc_core::{self, IoHandler};
use std::{
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
    thread::sleep,
    time::Duration,
};
#[cfg(test)]
use test_utils::mock_signing::mock_conductor_api;

/// Context holds the components that parts of a Holochain instance need in order to operate.
/// This includes components that are injected from the outside like logger and persister
/// but also the store of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct Context {
    pub agent_id: AgentId,
    pub logger: Arc<Mutex<dyn Logger>>,
    pub persister: Arc<Mutex<dyn Persister>>,
    state: Option<Arc<RwLock<StateWrapper>>>,
    pub action_channel: Option<Sender<ActionWrapper>>,
    pub observer_channel: Option<Sender<Observer>>,
    pub chain_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    pub dht_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    pub eav_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
    pub p2p_config: P2pConfig,
    pub conductor_api: ConductorApi,
    pub(crate) signal_tx: Option<Sender<Signal>>,
    pub(crate) instance_is_alive: Arc<Mutex<bool>>,
}

impl Context {
    // test_check_conductor_api() is used to inject a conductor_api with a working
    // mock of agent/sign to be used in tests.
    // There are two different implementations of this function below which get pulled
    // in depending on if "test" is in the build config, or not.
    // This allows unit tests of core to not have to deal with a conductor_api.
    #[cfg(not(test))]
    fn test_check_conductor_api(
        conductor_api: Option<Arc<RwLock<IoHandler>>>,
        _agent_id: AgentId,
    ) -> Arc<RwLock<IoHandler>> {
        // If you get here through this panic make sure that the context passed into the instance
        // gets created with a real conductor API. In test config it will be populated with mock API
        // that implements agent/sign with the mock_signer. We need this for testing but should
        // never use that code in production!
        // Hence the two different cases here.
        conductor_api.expect("Context can't be created without conductor API")
    }

    #[cfg(test)]
    fn test_check_conductor_api(
        conductor_api: Option<Arc<RwLock<IoHandler>>>,
        agent_id: AgentId,
    ) -> Arc<RwLock<IoHandler>> {
        conductor_api.unwrap_or_else(|| Arc::new(RwLock::new(mock_conductor_api(agent_id))))
    }

    pub fn new(
        agent_id: AgentId,
        logger: Arc<Mutex<dyn Logger>>,
        persister: Arc<Mutex<dyn Persister>>,
        chain_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        dht_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        eav: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
        p2p_config: P2pConfig,
        conductor_api: Option<Arc<RwLock<IoHandler>>>,
        signal_tx: Option<SignalSender>,
    ) -> Self {
        Context {
            agent_id: agent_id.clone(),
            logger,
            persister,
            state: None,
            action_channel: None,
            signal_tx,
            observer_channel: None,
            chain_storage,
            dht_storage,
            eav_storage: eav,
            p2p_config,
            conductor_api: ConductorApi::new(Self::test_check_conductor_api(
                conductor_api,
                agent_id,
            )),
            instance_is_alive: Arc::new(Mutex::new(true)),
        }
    }

    pub fn new_with_channels(
        agent_id: AgentId,
        logger: Arc<Mutex<dyn Logger>>,
        persister: Arc<Mutex<dyn Persister>>,
        action_channel: Option<Sender<ActionWrapper>>,
        signal_tx: Option<Sender<Signal>>,
        observer_channel: Option<Sender<Observer>>,
        cas: Arc<RwLock<dyn ContentAddressableStorage>>,
        eav: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
        p2p_config: P2pConfig,
    ) -> Result<Context, HolochainError> {
        Ok(Context {
            agent_id: agent_id.clone(),
            logger,
            persister,
            state: None,
            action_channel,
            signal_tx,
            observer_channel,
            chain_storage: cas.clone(),
            dht_storage: cas,
            eav_storage: eav,
            p2p_config,
            conductor_api: ConductorApi::new(Self::test_check_conductor_api(None, agent_id)),
            instance_is_alive: Arc::new(Mutex::new(true)),
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

    pub fn set_state(&mut self, state: Arc<RwLock<StateWrapper>>) {
        self.state = Some(state);
    }

    pub fn state(&self) -> Option<RwLockReadGuard<StateWrapper>> {
        self.state.as_ref().map(|s| s.read().unwrap())
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
            .cloned()
            .filter(|wasm| !wasm.code.is_empty())
    }

    // @NB: these three getters smell bad because previously Instance and Context had SyncSenders
    // rather than Option<SyncSenders>, but these would be initialized by default to broken channels
    // which would panic if `send` was called upon them. These `expect`s just bring more visibility to
    // that potential failure mode.
    // @see https://github.com/holochain/holochain-rust/issues/739
    pub fn action_channel(&self) -> &Sender<ActionWrapper> {
        self.action_channel
            .as_ref()
            .expect("Action channel not initialized")
    }

    pub fn is_action_channel_open(&self) -> bool {
        self.action_channel
            .clone()
            .map(|tx| tx.send(ActionWrapper::new(Action::Ping)).is_ok())
            .unwrap_or(false)
    }

    pub fn action_channel_error(&self, msg: &str) -> Option<HolochainError> {
        match &self.action_channel {
            Some(tx) => match tx.send(ActionWrapper::new(Action::Ping)) {
                Ok(()) => None,
                Err(_) => Some(HolochainError::LifecycleError(msg.into())),
            },
            None => Some(HolochainError::InitializationFailed(msg.into())),
        }
    }

    pub fn signal_tx(&self) -> Option<&Sender<Signal>> {
        self.signal_tx.as_ref()
    }

    pub fn observer_channel(&self) -> &Sender<Observer> {
        self.observer_channel
            .as_ref()
            .expect("Observer channel not initialized")
    }

    pub fn instance_still_alive(&self) -> bool {
        *self.instance_is_alive.lock().unwrap()
    }

    /// This creates an observer for the instance's redux loop and installs it.
    /// The returned receiver gets sent ticks from the instance every time the state
    /// got mutated.
    /// This enables blocking/parking the calling thread until the application state got changed.
    pub fn create_observer(&self) -> Receiver<()> {
        let (tick_tx, tick_rx) = unbounded();
        self.observer_channel()
            .send(Observer { ticker: tick_tx })
            .expect("Observer channel not initialized");
        tick_rx
    }

    /// Custom future executor that enables nested futures and nested calls of `block_on`.
    /// This makes use of the redux action loop and the observers.
    /// The given future gets polled everytime the instance's state got changed.
    pub fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        let tick_rx = self.create_observer();
        pin_utils::pin_mut!(future);

        let mut context = std::task::Context::from_waker(noop_waker_ref());

        loop {
            let _ = match future.as_mut().poll(&mut context) {
                Poll::Ready(result) => return result,
                _ => tick_rx.recv_timeout(Duration::from_millis(10)),
            };
            if !self.instance_still_alive() {
                panic!("Context::block_on() waiting for future but instance is not alive anymore => we gotta let this thread panic!")
            }
            if let Some(err) = self.action_channel_error("Context::block_on") {
                panic!("Context::block_on() waiting for future but Redux loop got stopped => we gotta let this thread panic!\nError was: {:?}", err)
            }
        }
    }

    /// returns the public capability token (if any)
    pub fn get_public_token(&self) -> Result<Address, HolochainError> {
        let state = self.state().ok_or("State uninitialized!")?;
        let top = state
            .agent()
            .top_chain_header()
            .ok_or::<HolochainError>("No top chain header".into())?;

        // Get address of first Token Grant entry (return early if none)
        let grants = state
            .agent()
            .chain_store()
            .iter_type(&Some(top), &EntryType::CapTokenGrant);

        // Get CAS
        let cas = state.agent().chain_store().content_storage();

        for grant in grants {
            let addr = grant.entry_address().to_owned();
            let entry = get_entry_from_cas(&cas, &addr)?
                .ok_or::<HolochainError>("Can't get CapTokenGrant entry from CAS".into())?;
            // if entry is the public grant return it
            if let Entry::CapTokenGrant(grant) = entry {
                if grant.cap_type() == CapabilityType::Public
                    && grant.id() == ReservedCapabilityId::Public.as_str()
                {
                    return Ok(addr);
                }
            }
        }

        Err(HolochainError::ErrorGeneric(
            "No public CapTokenGrant entry type in chain".into(),
        ))
    }
}

pub async fn get_dna_and_agent(context: &Arc<Context>) -> HcResult<(Address, String)> {
    let state = context
        .state()
        .ok_or("Network::start() could not get application state".to_string())?;
    let agent_state = state.agent();
    let agent = agent_state.get_agent()?;
    let agent_id = agent.pub_sign_key;

    let dna = state
        .nucleus()
        .dna()
        .ok_or("Network::start() called without DNA".to_string())?;
    Ok((dna.address(), agent_id))
}

/// Create an in-memory network config with the provided name,
/// otherwise create a unique name and thus network using snowflake.
/// This is the base function that many other `text_context*` functions use, and hence they also
/// require an optional network name. The reasoning for this is that tests which only require a
/// single instance may simply pass None and get a unique network name, but tests which require two
/// instances to be on the same network need to ensure both contexts use the same network name.
#[cfg_attr(tarpaulin, skip)]
pub fn test_memory_network_config(network_name: Option<&str>) -> P2pConfig {
    network_name
        .map(|name| P2pConfig::new_with_memory_backend(name))
        .unwrap_or(P2pConfig::new_with_unique_memory_backend())
}

#[cfg(test)]
pub mod tests {
    use self::tempfile::tempdir;
    use super::*;
    use crate::{logger::test_logger, persister::SimplePersister};
    use holochain_core_types::agent::AgentId;
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use std::sync::{Arc, Mutex, RwLock};
    use tempfile;

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
            P2pConfig::new_with_unique_memory_backend(),
            None,
            None,
        );

        assert!(maybe_context.state().is_none());

        let global_state = Arc::new(RwLock::new(StateWrapper::new(Arc::new(
            maybe_context.clone(),
        ))));
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
            P2pConfig::new_with_unique_memory_backend(),
            None,
            None,
        );

        let global_state = Arc::new(RwLock::new(StateWrapper::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());

        {
            let _write_lock = global_state.write().unwrap();
            // This line panics because we would enter into a deadlock
            context.state();
        }
    }
}
