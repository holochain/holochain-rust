use boolinator::*;
use crate::{
    action::ActionWrapper,
    context::Context,
    network::state::NetworkState,
};
use holochain_core_types::{
    cas::content::{Address},
    error::HolochainError,
};
use holochain_net_connection::{
    net_connection::NetConnection,
    protocol_wrapper::{GetDhtData, ProtocolWrapper},
};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    address: &Address,
) -> Result<(), HolochainError> {
    (network_state.network.is_some()
        && network_state.dna_hash.is_some() & network_state.agent_id.is_some())
        .ok_or("Network not initialized".to_string())?;

    let data = GetDhtData {
        msg_id: "?".to_string(),
        dna_hash: network_state.dna_hash.clone().unwrap(),
        from_agent_id: network_state.agent_id.clone().unwrap(),
        address: address.to_string(),
    };

    network_state
        .network
        .as_mut()
        .map(|network| {
            network
                .lock()
                .unwrap()
                .send(ProtocolWrapper::GetDht(data).into())
                .map_err(|error| HolochainError::IoError(error.to_string()))
        })
        .expect("Network has to be Some because of check above")
}

pub fn reduce_get_entry(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::GetEntry);

    let result = match inner(network_state, &address) {
        Ok(()) => None,
        Err(err) => Some(Err(err))
    };

    println!("ADR: {}", address);

    network_state.get_entry_results.insert(address.clone(), result);
}

#[cfg(test)]
mod tests {

    use crate::{
        context::mock_network_config,
        action::{Action, ActionWrapper},
        instance::tests::test_context,
        state::test_store,
    };
    use holochain_core_types::error::HolochainError;

    #[test]
    pub fn reduce_get_entry_without_network_initialized() {
        let context = test_context("alice");
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::GetEntry(entry.address()));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store.network().get_entry_results.get(&entry.address())
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(Some(Err(HolochainError::ErrorGeneric("Network not initialized".to_string())))));
    }

    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};

    #[test]
    pub fn reduce_get_entry_test() {
        let context = test_context("alice");
        let store = test_store(context.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork((mock_network_config(), String::from("abcd"), String::from("abcd"))));
        let store = store.reduce(context.clone(), action_wrapper);

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::GetEntry(entry.address()));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store.network().get_entry_results.get(&entry.address())
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(None));
    }

}
