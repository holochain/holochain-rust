use crate::{
    action::ActionWrapper,
    context::Context,
    network::{direct_message::DirectMessage, reducers::send_message, state::NetworkState},
};
use holochain_core_types::{chain_header::ChainHeader, error::HolochainError};
use std::sync::Arc;

fn inner(network_state: &mut NetworkState, header: &ChainHeader) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let source_address = header
        .sources()
        .first()
        .ok_or(HolochainError::ErrorGeneric(
            "No source found in ChainHeader".to_string(),
        ))?;
    let direct_message = DirectMessage::RequestValidationPackage(header.entry_address().clone());

    send_message(network_state, source_address, direct_message)
}

pub fn reduce_get_validation_package(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let header = unwrap_to!(action => crate::action::Action::GetValidationPackage);
    let entry_address = header.entry_address().clone();

    let result = match inner(network_state, header) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_validation_package_results
        .insert(entry_address, result);
}
