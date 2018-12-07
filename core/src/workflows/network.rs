use crate::{
    context::{Context,get_dna_and_agent},
    network::actions::initialize_network
};

use std::{
    sync::Arc,
};

use holochain_core_types::error::HcResult;


pub async fn initialize(context:Arc<Context>) -> HcResult<()>
{
    let agent_dna_pair = await!(get_dna_and_agent(&context))?;
    if agent_dna_pair.0.is_empty() && agent_dna_pair.1.is_empty()
    {
        Ok(await!(initialize_network::initialize_network(&context))?)
    }
    else
    {
        Ok(())
    }

}