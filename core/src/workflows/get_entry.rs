use crate::{context::Context, network, nucleus};

use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
use std::sync::Arc;

pub async fn get_entry<'a>(
    context: &'a Arc<Context>,
    address: &'a Address,
) -> Result<Option<Entry>, HolochainError> {
    let maybe_local_entry = await!(nucleus::actions::get_entry::get_entry(
        context,
        address.clone()
    ))?;
    if maybe_local_entry.is_some() {
        Ok(maybe_local_entry)
    } else {
        await!(network::actions::get_entry::get_entry(
            address.clone(),
            context
        ))
    }
}
