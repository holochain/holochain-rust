use crate::{
    context::Context,
    nucleus::ribosome::callback::{init::init, CallbackParams, CallbackResult},
};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
};
use std::sync::Arc;

/// Creates a network proxy object and stores DNA and agent hash in the network state.
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn call_init(dna: Dna, context: &Arc<Context>) -> HcResult<()> {
    // map init across every zome. Find which zomes init callback errored, if any
    let errors: Vec<(String, String)> = dna
        .zomes
        .keys()
        .map(|zome_name| {
            (
                zome_name,
                init(context.clone(), zome_name, &CallbackParams::Init),
            )
        })
        .filter_map(|(zome_name, result)| match result {
            CallbackResult::Fail(error_string) => Some((zome_name.to_owned(), error_string)),
            _ => None,
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HolochainError::ErrorGeneric(format!(
            "At least one zome init returned error: {:?}",
            errors
        )))
    }
}
