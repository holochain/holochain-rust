use crate::{
    context::{Context,get_dna_and_agent},
    network::actions::initialize_network
};

use std::{
    sync::Arc,
};

use holochain_core_types::{error::{HcResult,HolochainError},dna::Dna};
use futures::executor::block_on;
use crate::nucleus::actions::initialize::initialize_application;


pub async fn initialize(dna:Option<Dna>,context:Arc<Context>) -> HcResult<()>
{
    
    match await!(get_dna_and_agent(&context))
    {
        Ok(_) =>{
            println!("get dna");
            ()
        },
        Err(_) =>{
            println!("initialize application");
            await!(initialize_application(Dna::new(), &context)).unwrap();
        }
    };
    await!(initialize_network::initialize_network(&context))?;
    Ok(())
}