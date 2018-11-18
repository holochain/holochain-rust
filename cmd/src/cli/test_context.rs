use tempfile::tempdir;

use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::memory::EavMemoryStorage,
};
use holochain_core::{context::Context, logger::Logger, persister::SimplePersister};
use holochain_core_types::entry::agent::Agent;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, Debug)]
pub struct TestLogger {
    pub log: Vec<String>,
}

impl Logger for TestLogger {
    fn log(&mut self, msg: String) {
        self.log.push(msg);
    }
    fn dump(&self) -> String {
        format!("{:?}", self.log)
    }
}

/// create a test logger
pub fn test_logger() -> Arc<Mutex<TestLogger>> {
    Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
}

/// create a test context and TestLogger pair so we can use the logger in assertions
#[cfg_attr(tarpaulin, skip)]
pub fn test_context(agent_name: &str) -> Arc<Context> {
    let tempdir = tempdir().unwrap();
    let agent = Agent::generate_fake(agent_name);
    let logger = test_logger();
    let file_storage = Arc::new(RwLock::new(
        FilesystemStorage::new(tempdir.path().to_str().unwrap()).unwrap(),
    ));
    Arc::new(
        Context::new(
            agent,
            logger.clone(),
            Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
            Arc::new(RwLock::new(MemoryStorage::new().unwrap())),
            Arc::new(RwLock::new(EavMemoryStorage::new().unwrap())),
        ).unwrap(),
    )
}
