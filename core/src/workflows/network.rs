use crate::{
    context::{Context,get_dna_and_agent},
    network::actions::initialize_network
};

use std::{
    sync::Arc,
};

use holochain_core_types::{error::{HcResult,HolochainError},dna::Dna};
use futures::{executor::block_on,TryFutureExt,FutureExt};
use crate::{nucleus::actions::initialize::initialize_application,instance::Instance};


pub async fn initialize(instance:&Instance,dna:Option<Dna>,context:Arc<Context>) -> HcResult<Arc<Context>>
{
    let instance_context = instance.initialize_context(context.clone());
    await!(get_dna_and_agent(&instance_context)
    .map_ok(|_|{()})
    .or_else(|_|{
        initialize_application(dna.unwrap_or(Dna::new()), &instance_context).map_ok(|_|{()})
    }))?;    
    await!(initialize_network::initialize_network(&instance_context))?;
    Ok(instance_context)
}