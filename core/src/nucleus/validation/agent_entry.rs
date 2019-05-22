use crate::{
    context::Context,
    nucleus::{
        actions::{
            run_validation_callback::run_validation_callback,
        },
        validation::{
            ValidationResult
        },
        CallbackFnCall,
    },
};
use holochain_core_types::{
    entry::Entry,
    json::JsonString,
    cas::content::AddressableContent,
    validation::ValidationData,
};
use std::sync::Arc;
use futures::future;
use futures_util::future::FutureExt;

pub async fn validate_agent_entry(
    entry: Entry,
    _validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let dna = context.get_dna().expect("Callback called without DNA set!");

    let _results = await!(future::join_all(
        dna.zomes
        .iter()
        .map(|(zome_name, _)| {
            let call = CallbackFnCall::new(&zome_name, "__hdk_validate_agent_entry", JsonString::empty_object());
            run_validation_callback(entry.address(), call, &context).boxed()
        })
    ));

    Ok(())
}
