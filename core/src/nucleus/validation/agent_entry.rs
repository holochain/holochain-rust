use crate::{
    context::Context,
    nucleus::{
        validation::{ValidationResult},
    },
};
use holochain_core_types::{
    entry::{Entry},
    validation::ValidationData,
};
use std::sync::Arc;

pub async fn validate_agent_entry(
    _entry: Entry,
    _validation_data: ValidationData,
    _context: &Arc<Context>,
) -> ValidationResult {
    Ok(())
}
