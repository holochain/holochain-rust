use crate::{
    context::{get_dna_and_agent, ContextOnly, ContextStateful},
    network::actions::initialize_network,
};

use std::sync::Arc;

use crate::{instance::Instance, nucleus::actions::initialize::initialize_application};
use futures::TryFutureExt;
use holochain_core_types::{dna::Dna, error::HcResult};

pub async fn initialize(
    instance: &Instance,
    dna: Option<Dna>,
    mut context: Arc<ContextOnly>,
) -> HcResult<Arc<ContextStateful>> {
    let instance_context = instance.initialize_context(&mut context);
    println!("hmmmmm 5.1.1");
    await!(get_dna_and_agent(&instance_context)
        .map_ok(|_| ())
        .or_else(
            |_| initialize_application(dna.unwrap_or(Dna::new()), &instance_context).map_ok(|_| ())
        ))?;
    println!("hmmmmm 5.1.2");
    await!(initialize_network::initialize_network(&instance_context))?;
    Ok(instance_context)
}
