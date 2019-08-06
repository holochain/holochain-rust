use crate::{
    context::{get_dna_and_agent, Context},
    instance::Instance,
    network::actions::initialize_network::initialize_network,
    nucleus::actions::{call_init::call_init, initialize::initialize_chain},
};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
};

use std::sync::Arc;

pub async fn initialize(
    instance: &Instance,
    dna: Option<Dna>,
    context: Arc<Context>,
) -> HcResult<Arc<Context>> {
    let instance_context = instance.initialize_context(context.clone());
    let dna = dna.ok_or(HolochainError::DnaMissing)?;
    if let Err(err) = await!(get_dna_and_agent(&instance_context)) {
        log_warn!(context,
            "dna/initialize: Couldn't get DNA and agent from chain: {:?}",
            err
        );
        log_info!(context, "dna/initialize: Initializing new chain from given DNA...");
        await!(initialize_chain(dna.clone(), &instance_context))?;
    }
    await!(initialize_network(&instance_context))?;
    await!(call_init(dna, &instance_context))?;
    Ok(instance_context)
}
