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

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn initialize(
    instance: &Instance,
    maybe_dna: Option<Dna>,
    context: Arc<Context>,
) -> HcResult<Arc<Context>> {
    let instance_context = instance.initialize_context(context.clone());

    // This function is called in two different cases:
    // 1. Initializing a brand new instance
    // 2. Loading a persistent instance from storage
    //
    // In the first case, we definitely need maybe_dna to be Some, because that is the DNA
    // that will seed this instance's Nucleus.
    // If maybe_dna is None, we expect to find a DNA in the Nucleus (=in the state) already.
    let dna = if let Some(dna) = maybe_dna {
        // Ok, since maybe_dna is set, we are assuming to seed a new instance.
        // To make sure that we are not running into a weird state, we are going
        // to check here if we really deal with a fresh state and no DNA in the Nucleus already:
        if context.get_dna().is_some() {
            panic!("Tried to initialize instance that already has a DNA in its Nucleus");
        }
        dna
    } else {
        // No DNA provided as parameter.
        // This is the loading-case - we assume to find a DNA in the Nucleus state:
        context.get_dna().ok_or_else(|| {
            log_error!(
                context,
                "No DNA provided during loading and none found in state"
            );
            HolochainError::DnaMissing
        })?
    };

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
