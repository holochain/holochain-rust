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
    let (agent_id,dna_id) = await!(get_dna_and_agent(&context))?;
    if agent_id.is_empty() && dna_id.is_empty()
    {
        Ok(await!(initialize_network::initialize_network(&context))?)
    }
    else
    {
        Ok(())
    }

}