use crate::{
    action::{ActionWrapper, ValidationKey},
    network::{direct_message::DirectMessage, reducers::send_message, state::NetworkState},
    state::State,
};
use holochain_core_types::{chain_header::ChainHeader, error::HolochainError};
use std::time::{Duration, SystemTime};

// Some thought needs to go in to how long this should really be
// Should probably also be configurable via config or env vars
const GET_VALIDATION_PACKAGE_MESSAGE_TIMEOUT_MS: u64 = 61000;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn inner(
    network_state: &mut NetworkState,
    header: &ChainHeader,
    key: ValidationKey,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    let source_address = &header
        .provenances()
        .first()
        .ok_or_else(|| HolochainError::ErrorGeneric("No source found in ChainHeader".to_string()))?
        .source();
    let direct_message = DirectMessage::RequestValidationPackage(key);
    send_message(network_state, source_address, direct_message)
}
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_get_validation_package(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (key, header) = unwrap_to!(action => crate::action::Action::GetValidationPackage);

    let result = match inner(network_state, header, key.clone()) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_validation_package_results
        .insert(key.clone(), result);

    let timeout = (
        SystemTime::now(),
        Duration::from_millis(GET_VALIDATION_PACKAGE_MESSAGE_TIMEOUT_MS),
    );
    tracing::debug!(new_val_pack = ?key);
    network_state
        .get_validation_package_timeouts
        .insert(key.clone(), timeout);
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_get_validation_package_timeout(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => crate::action::Action::GetValidationPackageTimeout);

    network_state.get_validation_package_timeouts.remove(key);

    if let Some(Some(_)) = network_state.get_validation_package_results.get(key) {
        // A result already came back from the network so don't overwrite it
        return;
    }

    network_state.get_validation_package_results.insert(
        key.clone(),
        Some(Err(HolochainError::Timeout(format!(
            "timeout src: {}:{}",
            file!(),
            line!()
        )))),
    );
}
