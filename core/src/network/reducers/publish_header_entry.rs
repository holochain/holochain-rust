use crate::{
    action::ActionWrapper,
    network::{
        actions::ActionResponse,
        entry_aspect::EntryAspect,
        entry_with_header::{fetch_entry_with_header},
        reducers::send,
        state::NetworkState,
    },
    state::State,
    agent::state::create_new_chain_header,
};
use holochain_core_types::{
    entry::{Entry},
    error::HolochainError,
    chain_header::ChainHeader,
};
use lib3h_protocol::{
    data_types::{EntryData, ProvidedEntryData},
    protocol_client::Lib3hClientProtocol,
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};
use crate::state::StateWrapper;


/// Send to network a request to publish a header entry alone
/// This is similar to publishing a regular entry but it has its own special dummy header.
fn publish_header(
    network_state: &mut NetworkState,
    root_state: &State,
    chain_header: &ChainHeader,
) -> Result<(), HolochainError> {
    let header_entry = Entry::ChainHeader(chain_header.clone());
    let header_entry_header = create_new_chain_header(
        &header_entry,
        &root_state.agent(),
        &StateWrapper::from(root_state.clone()),
        &None,
        &Vec::new(),
    )?;
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: header_entry.address().clone(),
                aspect_list: vec![EntryAspect::Content(
                    header_entry.clone(),
                    header_entry_header,
                )
                .into()],
            },
        }),
    )
}


fn reduce_publish_header_entry_inner(
    network_state: &mut NetworkState,
    root_state: &State,
    address: &Address,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let entry_with_header = fetch_entry_with_header(&address, root_state)?;
    publish_header(network_state, root_state, &entry_with_header.header)
}

pub fn reduce_publish_header_entry(
    network_state: &mut NetworkState,
    root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::PublishHeaderEntry);

    let result = reduce_publish_header_entry_inner(network_state, root_state, &address);
    network_state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::PublishHeaderEntry(match result {
            Ok(_) => Ok(address.clone()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        }),
    );
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper},
        instance::tests::test_context,
        state::test_store,
    };
    use holochain_core_types::entry::test_entry;
    use holochain_persistence_api::cas::content::AddressableContent;

    #[test]
    pub fn reduce_publish_header_entry_test() {
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let action_wrapper = ActionWrapper::new(Action::PublishHeaderEntry(entry.address()));

        store.reduce(action_wrapper);
    }
}
