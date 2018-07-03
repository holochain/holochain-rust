use error::HolochainError;
use holochain_agent::Agent;
use logger::Logger;
use persister::Persister;
use std::sync::{Arc, Mutex};

/// Context holds those aspects of the outside world that a Holochain instance needs to operate
#[derive(Clone)]
pub struct Context {
    pub agent: Agent,
    pub logger: Arc<Mutex<Logger>>,
    pub persister: Arc<Mutex<Persister>>,
}

impl Context {
    // helper function to make it easier to call the logger
    pub fn log(&self, msg: &str) -> Result<(), HolochainError> {
        let result = self.logger.lock();
        match result {
            Err(_) => return Err(HolochainError::LoggingError),
            Ok(mut logger) => {
                logger.log(msg.to_string());
            }
        }
        Ok(())
    }
}
