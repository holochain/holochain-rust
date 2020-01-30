use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },NEW_RELIC_LICENSE_KEY
};
use holochain_core_types::{
    agent::AgentId,
    entry::Entry,
    validation::{EntryValidationData, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use holochain_wasm_utils::api_serialization::validation::AgentIdValidationArgs;

use futures::{future, future::FutureExt};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_agent_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    let agent_id = unwrap_to!(entry => Entry::AgentId);

    let params = AgentIdValidationArgs {
        validation_data: EntryValidationData::<AgentId>::Create {
            entry: agent_id.to_owned(),
            validation_data,
        },
    };

    log_debug!(context, "Validating agent entry with args: {:?}", params);

    let results = future::join_all(dna.zomes.iter().map(|(zome_name, _)| {
        let call = CallbackFnCall::new(&zome_name, "__hdk_validate_agent_entry", params.clone());
        // Need to return a boxed future for it to work with join_all
        // https://users.rust-lang.org/t/the-trait-unpin-is-not-implemented-for-genfuture-error-when-using-join-all/23612/2
        run_validation_callback(entry.address(), call, &context).boxed()
    }))
    .await;

    let errors: Vec<ValidationError> = results
        .iter()
        .filter_map(|r| match r {
            Ok(_) => None,
            Err(e) => Some(e.to_owned()),
        })
        .collect();

    if errors.is_empty() {
        log_debug!(context, "Validating agent entry success!: {:?}", results);
        Ok(())
    } else {
        Err(ValidationError::Error(
            format!("Failed to validate agent ID on a zome, {:?}", errors).into(),
        ))
    }
}
