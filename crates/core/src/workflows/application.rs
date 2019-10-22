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

    // 2. Initialize the local chain if not already
    let first_initialization = match get_dna_and_agent(&instance_context).await {
        Ok(_) => false,
        Err(err) => {
            log_debug!(
                context,
                "dna/initialize: No DNA and agent in chain so assuming uninitialized: {:?}",
                err
            );
            initialize_chain(dna.clone(), &instance_context).await?;
            log_debug!(
                context,
                "dna/initialize: Initializing new chain from given DNA..."
            );
            true
        }
    };

    // 3. Initialize the network
    initialize_network(&instance_context).await?;

    // 4. (first initialization only) Call the init callbacks in the zomes
    if first_initialization {
        call_init(dna, &instance_context).await?;
    }
    Ok(instance_context)
}
