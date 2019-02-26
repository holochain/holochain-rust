use crate::{
    context::Context,
    dht::actions::{crud_link::crud_link as crud_link_action_creator, update_entry::update_entry,remove_entry::remove_entry},
};

use holochain_core_types::{cas::content::Address, crud_status::CrudStatus, error::HolochainError};
use std::sync::Arc;

pub async fn crud_status_workflow<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
    crud_status: &'a CrudStatus,
) -> Result<(), HolochainError> {
    //create crud_status passed in
    println!("crud workflow {:?}", crud_status.clone());
    match crud_status {
        CrudStatus::Modified => await!(update_crud_status(context, address)),
        CrudStatus::Deleted => await!(remove_crud_status(context, address)),
        CrudStatus::Live => Ok(()),
        _ => Err(HolochainError::NotImplemented(
            "Crud Status Not Implemented".to_string(),
        )),
    }
}

pub async fn crud_link_workflow<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
    crud_link: &'a Option<Address>,
) -> Result<(), HolochainError> {
    let link = crud_link.clone().ok_or(HolochainError::ErrorGeneric(
        "Could not get crud link".to_string(),
    ))?;
    await!(crud_link_action_creator(context, address.clone(), link)?)?;
    Ok(())
}

async fn update_crud_status<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
) -> Result<(), HolochainError> {
    await!(update_entry(context, address.clone())?)?;
    Ok(())
}

async fn remove_crud_status<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
) -> Result<(), HolochainError> {
    await!(remove_entry(context, address.clone())?)?;
    Ok(())
}
