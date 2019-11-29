use crate::{
    action::{Action, ActionWrapper},
    content_store::GetContent,
    instance::Observer,
    network::state::NetworkState,
    persister::Persister,
    signal::{Signal, SignalSender},
    state::StateWrapper,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::{
    task::{noop_waker_ref, Poll},
    Future,
};
use holochain_conductor_lib_api::ConductorApi;
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
use holochain_locksmith::{Mutex, MutexGuard, RwLock, RwLockReadGuard};
use holochain_metrics::MetricPublisher;
use holochain_net::{p2p_config::P2pConfig, p2p_network::P2pNetwork};
use holochain_persistence_api::{
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    eav::EntityAttributeValueStorage,
};
use holochain_tracing as ht;
use jsonrpc_core::{self, IoHandler};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use futures::executor::ThreadPool;
#[cfg(test)]
use test_utils::mock_signing::mock_conductor_api;

pub type ActionSender = ht::SpanSender<ActionWrapper>;
pub type ActionReceiver = ht::SpanReceiver<ActionWrapper>;

pub struct P2pNetworkWrapper(Arc<Mutex<Option<P2pNetwork>>>);

impl P2pNetworkWrapper {
    pub fn lock(&self) -> P2pNetworkMutexGuardWrapper<'_> {
        P2pNetworkMutexGuardWrapper(self.0.lock().expect("network accessible"))
    }
}

pub struct P2pNetworkMutexGuardWrapper<'a>(MutexGuard<'a, Option<P2pNetwork>>);

impl<'a> P2pNetworkMutexGuardWrapper<'a> {
    pub fn as_ref(&self) -> Result<&P2pNetwork, HolochainError> {
        match self.0.as_ref() {
            Some(s) => Ok(s),
            None => Err(HolochainError::ErrorGeneric("no network".into())),
        }
    }
}

/// Context holds the components that parts of a Holochain instance need in order to operate.
/// This includes components that are injected from the outside like persister
/// but also the store of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct Context {
    pub(crate) instance_name: String,
    pub agent_id: AgentId,
    pub persister: Arc<RwLock<dyn Persister>>,
    state: Option<Arc<RwLock<StateWrapper>>>,
    pub action_channel: Option<ActionSender>,
    pub observer_channel: Option<Sender<Observer>>,
    pub chain_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    pub dht_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
    pub eav_storage: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
    pub p2p_config: P2pConfig,
    pub conductor_api: ConductorApi,
    pub(crate) signal_tx: Option<Sender<Signal>>,
    pub(crate) instance_is_alive: Arc<AtomicBool>,
    pub state_dump_logging: bool,
    thread_pool: Arc<Mutex<ThreadPool>>,
    pub redux_wants_write: Arc<AtomicBool>,
    pub metric_publisher: Arc<RwLock<dyn MetricPublisher>>,
    pub tracer: Arc<ht::Tracer>,
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

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_name: &str,
        agent_id: AgentId,
        persister: Arc<RwLock<dyn Persister>>,
        chain_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        dht_storage: Arc<RwLock<dyn ContentAddressableStorage>>,
        eav: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
        p2p_config: P2pConfig,
        conductor_api: Option<Arc<RwLock<IoHandler>>>,
        signal_tx: Option<SignalSender>,
        state_dump_logging: bool,
        metric_publisher: Arc<RwLock<dyn MetricPublisher>>,
        tracer: Arc<ht::Tracer>,
    ) -> Self {
        Context {
            instance_name: instance_name.to_owned(),
            agent_id: agent_id.clone(),
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
            instance_is_alive: Arc::new(AtomicBool::new(true)),
            state_dump_logging,
            thread_pool: Arc::new(Mutex::new(
                ThreadPool::new().expect("Could not create thread pool for futures"),
            )),
            redux_wants_write: Arc::new(AtomicBool::new(false)),
            metric_publisher,
            tracer,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_channels(
        instance_name: &str,
        agent_id: AgentId,
        persister: Arc<RwLock<dyn Persister>>,
        action_channel: Option<ActionSender>,
        signal_tx: Option<Sender<Signal>>,
        observer_channel: Option<Sender<Observer>>,
        cas: Arc<RwLock<dyn ContentAddressableStorage>>,
        eav: Arc<RwLock<dyn EntityAttributeValueStorage<Attribute>>>,
        p2p_config: P2pConfig,
        state_dump_logging: bool,
        metric_publisher: Arc<RwLock<dyn MetricPublisher>>,
        tracer: Arc<ht::Tracer>,
    ) -> Result<Context, HolochainError> {
        Ok(Context {
            instance_name: instance_name.to_owned(),
            agent_id: agent_id.clone(),
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
            instance_is_alive: Arc::new(AtomicBool::new(true)),
            state_dump_logging,
            thread_pool: Arc::new(Mutex::new(
                ThreadPool::new().expect("Could not create thread pool for futures"),
            )),
            redux_wants_write: Arc::new(AtomicBool::new(false)),
            metric_publisher,
            tracer,
        })
    }

    /// Returns the name of this context instance.
    pub fn get_instance_name(&self) -> String {
        self.instance_name.clone()
    }

    pub fn set_state(&mut self, state: Arc<RwLock<StateWrapper>>) {
        self.state = Some(state);
    }

    #[autotrace]
    pub fn state(&self) -> Option<StateWrapper> {
        self.state.as_ref().map(|s| {
            while self.redux_wants_write.load(Relaxed) {
                std::thread::sleep(Duration::from_millis(1));
            }
            (*s.read().unwrap().annotate("Context::state")).clone()
        })
    }

    /// Try to acquire read-lock on the state.
    /// Returns immediately either with the lock or with None if the lock
    /// is occupied already.
    /// Also returns None if the context was not initialized with a state.
    #[autotrace]
    pub fn try_state(&self) -> Option<RwLockReadGuard<StateWrapper>> {
        if self.redux_wants_write.load(Relaxed) {
            None
        } else {
            self.state
                .as_ref()
                .and_then(|s| s.try_read())
                .map(|lock| lock.annotate("Context::try_state"))
        }
    }

    pub fn network_state(&self) -> Option<Arc<NetworkState>> {
        self.state().map(move |state| state.network())
    }

    #[autotrace]
    pub fn get_dna(&self) -> Option<Dna> {
        // In the case of init we encounter race conditions with regards to setting the DNA.
        // Init gets called asynchronously right after dispatching an action that sets the DNA in
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

    #[autotrace]
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
    pub fn action_channel(&self) -> &ActionSender {
        self.action_channel
            .as_ref()
            .expect("Action channel not initialized")
    }

    pub fn is_action_channel_open(&self) -> bool {
        self.action_channel
            .clone()
            .map(|tx| tx.send_wrapped(ActionWrapper::new(Action::Ping)).is_ok())
            .unwrap_or(false)
    }

    pub fn action_channel_error(&self, msg: &str) -> Option<HolochainError> {
        match &self.action_channel {
            Some(tx) => match tx.send_wrapped(ActionWrapper::new(Action::Ping)) {
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
        self.instance_is_alive.load(Relaxed)
    }

    pub fn reset_instance(&mut self) {
        self.instance_is_alive = Arc::new(AtomicBool::new(true));
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
    #[autotrace]
    pub fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        let tick_rx = self.create_observer();
        pin_utils::pin_mut!(future);

        let mut cx = std::task::Context::from_waker(noop_waker_ref());

        loop {
            let _ = match future.as_mut().poll(&mut cx) {
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

    pub fn spawn_task<Fut>(&self, f: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.thread_pool
            .lock()
            .expect("Couldn't get lock on Context::thread_pool")
            .run(f);
    }

    /// returns the public capability token (if any)
    #[autotrace]
    pub fn get_public_token(&self) -> Result<Address, HolochainError> {
        let state = self.state().ok_or("State uninitialized!")?;
        let top = state
            .agent()
            .top_chain_header()
            .ok_or_else(|| HolochainError::from("No top chain header"))?;

        // Get address of first Token Grant entry (return early if none)
        let grants = state
            .agent()
            .chain_store()
            .iter_type(&Some(top), &EntryType::CapTokenGrant);

        for grant in grants {
            let addr = grant.entry_address().to_owned();
            let entry =
                state.agent().chain_store().get(&addr)?.ok_or_else(|| {
                    HolochainError::from("Can't get CapTokenGrant entry from CAS")
                })?;
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

#[autotrace]
pub async fn get_dna_and_agent(context: &Arc<Context>) -> HcResult<(Address, String)> {
    let state = context
        .state()
        .ok_or_else(|| "Network::start() could not get application state".to_string())?;
    let agent_state = state.agent();
    let agent = agent_state.get_agent()?;
    let agent_id = agent.pub_sign_key;

    let dna = state
        .nucleus()
        .dna()
        .ok_or_else(|| "Network::start() called without DNA".to_string())?;
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
        .unwrap_or_else(P2pConfig::new_with_unique_memory_backend)
}

#[cfg(test)]
pub mod tests {
    use self::tempfile::tempdir;
    use super::*;
    use crate::persister::SimplePersister;
    use holochain_core_types::agent::AgentId;
    use holochain_locksmith::RwLock;
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use std::sync::Arc;
    use tempfile;

    #[test]
    fn context_log_macro_test_from_context() {
        use crate::*;

        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let ctx = Context::new(
            "LOG-TEST-ID",
            AgentId::generate_fake("Bilbo"),
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

        // Somehow we need to build our own logging instance for this test to show logs
        use holochain_logging::prelude::*;
        let guard = FastLoggerBuilder::new()
            .set_level_from_str("Trace")
            .build()
            .expect("Fail to init logger.");

        // Tests if the context logger can be customized by poassing a target value
        log_info!(target: "holochain-custom-log-target", "Custom target & '{}' log level.", "Info");

        // Tests if the context logger fills its target with the instance ID
        log_trace!(ctx, "'{}' log level with Context target.", "Trace");
        log_debug!(ctx, "'{}' log level with Context target.", "Debug");
        log_info!(ctx, "'{}' log level with Context target.", "Info");
        log_warn!(ctx, "'{}' log level with Context target.", "Warning");
        log_error!(ctx, "'{}' log level with Context target.", "Error");

        guard.flush();
    }
}
