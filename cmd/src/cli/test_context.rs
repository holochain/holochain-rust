use holochain_container_api::context_builder::ContextBuilder;
use holochain_core::{context::Context, logger::Logger};
use holochain_core_types::agent::AgentId;
use std::sync::{Arc, Mutex};

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
    let agent = AgentId::generate_fake(agent_name);
    Arc::new(
        ContextBuilder::new()
            .with_agent(agent)
            .with_logger(test_logger())
            .with_memory_storage()
            .spawn(),
    )
}
