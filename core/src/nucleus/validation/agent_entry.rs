use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use futures::future;
use futures_util::future::FutureExt;
use holochain_core_types::{
    cas::content::AddressableContent, entry::Entry, validation::ValidationData, agent::AgentId,
};
use holochain_wasm_utils::api_serialization::validation::AgentIdValidationArgs;
use std::sync::Arc;

pub async fn validate_agent_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    let params = AgentIdValidationArgs {
        agent_id: AgentId::generate_fake("TODO"),
        validation_data,
    };

    let results = await!(future::join_all(dna.zomes.iter().map(|(zome_name, _)| {
        let call = CallbackFnCall::new(
            &zome_name,
            "__hdk_validate_agent_entry",
            params.clone(),
        );
        // Need to return a boxed future for it to work with join_all
        // https://users.rust-lang.org/t/the-trait-unpin-is-not-implemented-for-genfuture-error-when-using-join-all/23612/2
        run_validation_callback(entry.address(), call, &context).boxed()
    })));

    let errors: Vec<ValidationError> = results
        .iter()
        .filter_map(|r| match r {
            Ok(_) => None,
            Err(e) => Some(e.to_owned()),
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::Error(
            "Failed to validate agent ID on a zome".into(),
        ))
    }
}
