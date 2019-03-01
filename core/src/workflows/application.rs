use crate::{
    context::{get_dna_and_agent, Context},
    network::actions::initialize_network,
};

use std::sync::Arc;

use crate::{instance::Instance, nucleus::actions::initialize::initialize_application};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
};

pub async fn initialize(
    instance: &Instance,
    dna: Option<Dna>,
    context: Arc<Context>,
) -> HcResult<Arc<Context>> {
    let instance_context = instance.initialize_context(context.clone());
    if let Err(err) = await!(get_dna_and_agent(&instance_context)) {
        context.log(format!(
            "application/initialize: Couldn't get DNA and agent from chain: {:?}",
            err
        ));
        let dna = dna.ok_or(HolochainError::DnaMissing)?;
        context.log("application/initialize: Initializing new chain from given DNA...");
        await!(initialize_application(dna, &instance_context))?;
    }
    await!(initialize_network::initialize_network(&instance_context))?;
    Ok(instance_context)
}
