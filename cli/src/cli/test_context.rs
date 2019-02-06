use holochain_conductor_api::context_builder::ContextBuilder;
use holochain_core::{context::Context, logger::test_logger};
use holochain_core_types::agent::AgentId;
use std::sync::Arc;

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
