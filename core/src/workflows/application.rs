use crate::{
    context::{get_dna_and_agent, Context},
    instance::Instance,
    network::actions::{
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
    // 1. Intitialize the context
    let instance_context = instance.initialize_context(context.clone());
    let dna = dna.ok_or(HolochainError::DnaMissing)?;

    // 2. Initialize the local chain
    if let Err(err) = await!(get_dna_and_agent(&instance_context)) {
        context.log_warn(format!(
            "dna/initialize: Couldn't get DNA and agent from chain: {:?}",
            err
        ));
        context.log("dna/initialize: Initializing new chain from given DNA...");
        await!(initialize_chain(dna.clone(), &instance_context))?;
    }

    // 3. Initialize the network
    await!(initialize_network(&instance_context))?;

    // 4. Call publish on the agent and DNA entries. 
    // This is to trigger the publishing of their headers not the entries themselves
    let dna_entry = Entry::Dna(Box::new(dna.clone()));
    await!(publish(dna_entry.address(), &context))?;
    let agent_id_entry = Entry::AgentId(context.agent_id.clone());
    await!(publish(agent_id_entry.address(), &context))?;

    // 5. Call the init callbacks in the zomes
    await!(call_init(dna, &instance_context))?;
    Ok(instance_context)
}
