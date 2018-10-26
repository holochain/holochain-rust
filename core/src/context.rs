use action::ActionWrapper;
use holochain_agent::Agent;
use holochain_core_types::error::HolochainError;
use instance::Observer;
use logger::Logger;
use persister::Persister;
use state::State;
use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex, RwLock, RwLockReadGuard,
};

use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};

/// Context holds the components that parts of a Holochain instance need in order to operate.
/// This includes components that are injected from the outside like logger and persister
/// but also the store of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct Context {
    pub agent: Agent,
    pub logger: Arc<Mutex<Logger>>,
    pub persister: Arc<Mutex<Persister>>,
    state: Option<Arc<RwLock<State>>>,
    pub action_channel: SyncSender<ActionWrapper>,
    pub observer_channel: SyncSender<Observer>,
    pub file_storage: FilesystemStorage,
    pub eav_storage: EavFileStorage,
}

impl Context {
    pub fn default_channel_buffer_size() -> usize {
        100
    }

    pub fn new(
        agent: Agent,
        logger: Arc<Mutex<Logger>>,
        persister: Arc<Mutex<Persister>>,
        cas: FilesystemStorage,
        eav: EavFileStorage,
    ) -> Result<Context, HolochainError> {
        let (tx_action, _) = sync_channel(Self::default_channel_buffer_size());
        let (tx_observer, _) = sync_channel(Self::default_channel_buffer_size());
        Ok(Context {
            agent,
            logger,
            persister,
            state: None,
            action_channel: tx_action,
            observer_channel: tx_observer,
            file_storage: cas,
            eav_storage: eav,
        })
    }

    pub fn new_with_channels(
        agent: Agent,
        logger: Arc<Mutex<Logger>>,
        persister: Arc<Mutex<Persister>>,
        action_channel: SyncSender<ActionWrapper>,
        observer_channel: SyncSender<Observer>,
        cas: FilesystemStorage,
        eav: EavFileStorage,
    ) -> Result<Context, HolochainError> {
        Ok(Context {
            agent,
            logger,
            persister,
            state: None,
            action_channel,
            observer_channel,
            file_storage: cas,
            eav_storage: eav,
        })
    }
    // helper function to make it easier to call the logger
    pub fn log(&self, msg: &str) -> Result<(), HolochainError> {
        let mut logger = self.logger.lock().or(Err(HolochainError::LoggingError))?;
        logger.log(msg.to_string());
        Ok(())
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
}

#[cfg(test)]
mod tests {
    extern crate holochain_agent;
    extern crate tempfile;
    extern crate test_utils;
    use self::tempfile::tempdir;
    use super::*;
    use instance::tests::test_logger;
    use persister::SimplePersister;
    use state::State;
    use std::sync::{Arc, Mutex};

    #[test]
    fn default_buffer_size_test() {
        assert_eq!(Context::default_channel_buffer_size(), 100);
    }

    #[test]
    fn test_state() {
        let mut maybe_context = Context::new(
            holochain_agent::Agent::from("Terence".to_string()),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
            EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
        ).unwrap();

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
    fn test_deadlock() {
        let mut context = Context::new(
            holochain_agent::Agent::from("Terence".to_string()),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
            EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
        ).unwrap();

        let global_state = Arc::new(RwLock::new(State::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());

        {
            let _write_lock = global_state.write().unwrap();
            // This line panics because we would enter into a deadlock
            context.state();
        }
    }
}
