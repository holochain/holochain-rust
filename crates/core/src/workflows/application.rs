use crate::{
    context::{get_dna_and_agent, Context},
    instance::Instance,
    network::actions::{
        publish_header_entry::publish_header_entry,
        publish::publish,
        initialize_network::initialize_network,
    },
    nucleus::actions::{call_init::call_init, initialize::initialize_chain},
};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
    entry::Entry,
};
use holochain_persistence_api::cas::content::AddressableContent;
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
            log_debug!(context,
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

    if first_initialization {
        // 4. (first initialization only) Publish the agent entry and headers of the agent and DNA entries.
        publish(context.agent_id.address(), &context).await?;

        let dna_entry = Entry::Dna(Box::new(dna.clone()));
        publish_header_entry(dna_entry.address(), &context).await?;
        let agent_id_entry = Entry::AgentId(context.agent_id.clone());
        publish_header_entry(agent_id_entry.address(), &context).await?;

        // 5. (first initialization only) Call the init callbacks in the zomes
        call_init(dna, &instance_context).await?;
    }
    Ok(instance_context)
}
