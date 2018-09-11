use error::HolochainError;
use holochain_agent::Agent;
use logger::Logger;
use persister::Persister;
use state::State;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

/// Context holds those aspects of a living Holochain instance that components need to operate.
/// This includes modules that are injected from the outside like logger and persister
/// but also the state of the instance that gets injected before passing on the context
/// to inner components/reducers.
#[derive(Clone)]
pub struct Context {
    pub agent: Agent,
    pub logger: Arc<Mutex<Logger>>,
    pub persister: Arc<Mutex<Persister>>,
    state: Option<Arc<RwLock<State>>>,
}

impl Context {
    pub fn new(
        agent: Agent,
        logger: Arc<Mutex<Logger>>,
        persister: Arc<Mutex<Persister>>,
    ) -> Context {
        Context {
            agent,
            logger,
            persister,
            state: None,
        }
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
