use crate::{
    context::{get_dna_and_agent, Context},
    network::actions::initialize_network,
};

use std::sync::Arc;

use crate::{instance::Instance, nucleus::actions::initialize::initialize_application};
use futures::TryFutureExt;
use holochain_core_types::{dna::Dna, error::HcResult};

pub async fn initialize(
    instance: &Instance,
    dna: Option<Dna>,
    context: Arc<Context>,
) -> HcResult<Arc<Context>> {
    let instance_context = instance.initialize_context(context.clone());
    await!(get_dna_and_agent(&instance_context)
        .map_ok(|_| ())
        .or_else(|err| {
            context.log(format!(
                "application/initialize: Couldn't get DNA and agent from chain: {:?}",
                err
            ));
            context.log("application/initialize: Initializing new chain...");
            initialize_application(dna.unwrap_or(Dna::new()), &instance_context).map_ok(|_| ())
        }))?;
    await!(initialize_network::initialize_network(&instance_context))?;
    Ok(instance_context)
}
