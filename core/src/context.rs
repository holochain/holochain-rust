use crate::{
    action::ActionWrapper,
    instance::Observer,
    logger::Logger,
    persister::Persister,
    signal::{Signal, SignalSender},
    state::State,
};
use base64;
use futures::{
    task::{noop_local_waker_ref, Poll},
    Future,
};
use holochain_core_types::{
    agent::AgentId,
    cas::{
        content::{Address, AddressableContent},
        storage::ContentAddressableStorage,
    },
    dna::{wasm::DnaWasm, Dna},
    eav::EntityAttributeValueStorage,
    error::{HcResult, HolochainError},
};

use holochain_dpki::keypair::{Keypair, SEEDSIZE, SIGNATURESIZE};
use holochain_sodium::secbuf::SecBuf;

use holochain_net::p2p_config::P2pConfig;
use jsonrpc_lite::JsonRpc;
use jsonrpc_ws_server::jsonrpc_core::{self, types::params::Params, IoHandler};
use snowflake::ProcessUniqueId;
use std::{
    sync::{
        mpsc::{channel, Receiver, SyncSender},
        Arc, Mutex, RwLock, RwLockReadGuard,
    },
    thread::sleep,
    time::Duration,
};

/// This is a local mock for the `agent/sign` conductor API function.
/// It creates a syntactically equivalent signature using dpki::Keypair
/// but with key generated from a static/deterministic mock seed.
/// This enables unit testing of core code that creates signatures without
/// depending on the conductor or actual key files.
pub fn mock_signer(payload: String) -> String {
    // Create deterministic seed:
    let mut seed = SecBuf::with_insecure(SEEDSIZE);
    let mock_seed: Vec<u8> = (1..SEEDSIZE).map(|num| num as u8).collect();

    seed.write(0, mock_seed.as_slice())
        .expect("SecBuf must be writeable");

    // Create keypair from seed:
    let mut keypair = Keypair::new_from_seed(&mut seed).unwrap();

    // Convert payload string into a SecBuf
    let mut message = SecBuf::with_insecure_from_string(payload);

    // Create signature
    let mut message_signed = SecBuf::with_insecure(SIGNATURESIZE);
    keypair.sign(&mut message, &mut message_signed).unwrap();
    let message_signed = message_signed.read_lock();

    // Return as base64 encoded string
    base64::encode(&**message_signed)
}

/// Wraps `fn mock_signer(String) -> String` in an `IoHandler` to mock the conductor API
/// in a way that core can safely assume the conductor API to be present with at least
/// the `agent/sign` method.
fn mock_conductor_api() -> IoHandler {
    let mut handler = IoHandler::new();
    handler.add_method("agent/sign", move |params| {
        let params_map = match params {
            Params::Map(map) => Ok(map),
            _ => Err(jsonrpc_core::Error::invalid_params("expected params map")),
        }?;

        let key = "payload";
        let payload = Ok(params_map
            .get(key)
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` param not provided",
                key
            )))?
            .as_str()
            .ok_or(jsonrpc_core::Error::invalid_params(format!(
                "`{}` is not a valid json string",
                key
            )))?
            .to_string())?;

        Ok(json!({"payload": payload, "signature": mock_signer(payload)}))
    });
    handler
}

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
    pub p2p_config: P2pConfig,
    pub conductor_api: Arc<RwLock<IoHandler>>,
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
        p2p_config: P2pConfig,
        conductor_api: Option<Arc<RwLock<IoHandler>>>,
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
            p2p_config,
            conductor_api: conductor_api
                .or(Some(Arc::new(RwLock::new(mock_conductor_api()))))
                .unwrap(),
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
        p2p_config: P2pConfig,
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
            p2p_config,
            conductor_api: Arc::new(RwLock::new(mock_conductor_api())),
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
            .cloned()
            .filter(|wasm| !wasm.code.is_empty())
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

    /// This creates an observer for the instance's redux loop and installs it.
    /// The returned receiver gets sent ticks from the instance every time the state
    /// got mutated.
    /// This enables blocking/parking the calling thread until the application state got changed.
    pub fn create_observer(&self) -> Receiver<()> {
        let (tick_tx, tick_rx) = channel();
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

        loop {
            let _ = match future.as_mut().poll(noop_local_waker_ref()) {
                Poll::Ready(result) => return result,
                _ => tick_rx.recv_timeout(Duration::from_millis(10)),
            };
        }
    }

    pub fn sign(&self, payload: String) -> Result<String, HolochainError> {
        let handler = self.conductor_api.write().unwrap();
        let request = format!(
            r#"{{"jsonrpc": "2.0", "method": "agent/sign", "params": {{"payload": "{}"}}, "id": "{}"}}"#,
            payload, ProcessUniqueId::new()
        );

        let response = handler
            .handle_request_sync(&request)
            .ok_or("Conductor sign call failed".to_string())?;

        let response = JsonRpc::parse(&response)?;

        match response {
            JsonRpc::Success(_) => Ok(String::from(
                response.get_result().unwrap()["signature"]
                    .as_str()
                    .unwrap(),
            )),
            JsonRpc::Error(_) => Err(HolochainError::ErrorGeneric(
                serde_json::to_string(&response.get_error().unwrap()).unwrap(),
            )),
            _ => Err(HolochainError::ErrorGeneric(
                "Bridge call failed".to_string(),
            )),
        }
    }
}

pub async fn get_dna_and_agent(context: &Arc<Context>) -> HcResult<(Address, String)> {
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
    use crate::{logger::test_logger, persister::SimplePersister, state::State};
    use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use holochain_core_types::agent::AgentId;
    use std::sync::{Arc, Mutex, RwLock};
    use tempfile;

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
            P2pConfig::new_with_unique_memory_backend(),
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
            P2pConfig::new_with_unique_memory_backend(),
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
