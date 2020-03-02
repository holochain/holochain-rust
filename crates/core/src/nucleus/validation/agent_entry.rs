use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        CallbackFnCall,
    },
    
};
use holochain_core_types::{
    validation::{ValidationResult},
    agent::AgentId,
    entry::Entry,
    validation::{EntryValidationData, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use holochain_wasm_types::validation::AgentIdValidationArgs;

use futures::{future, future::FutureExt};
use std::sync::Arc;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_agent_entry(
    context: Arc<Context>,
    entry: Entry,
    validation_data: ValidationData,
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
        run_validation_callback(Arc::clone(&context), entry.address(), call).boxed()
    }))
    .await;

    let errors: Vec<ValidationResult> = results
        .iter()
        .filter_map(|r| match r {
            ValidationResult::Ok => None,
            v => Some(v.to_owned()),
        })
        .collect();

    if errors.is_empty() {
        log_debug!(context, "Validating agent entry success!: {:?}", results);
        ValidationResult::Ok
    } else {
        ValidationResult::Fail(
            format!("Failed to validate agent ID on a zome, {:?}", errors).into(),
        )
    }
}
