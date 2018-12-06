use crate::{
    context::Context,
    network::actions::initialize_network
};

use std::{
    sync::Arc,
};

use holochain_core_types::error::HolochainError;


pub async fn initialize(context:Arc<Context>) -> Result<(), HolochainError>
{
    let agent_dna = await!(initialize_network::get_dna_and_agent(&context))?;
    if agent_dna.0.is_empty() && agent_dna.1.is_empty()
    {
        Ok(await!(initialize_network::initialize_network(&context))?)
    }
    else
    {
        Ok(())
    }

}