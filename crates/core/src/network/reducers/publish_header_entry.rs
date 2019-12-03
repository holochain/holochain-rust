use crate::{
    action::ActionWrapper,
    agent::state::create_entry_with_header_for_header,
    network::{
        actions::NetworkActionResponse,
        entry_aspect::EntryAspect,
        entry_with_header::{fetch_entry_with_header, EntryWithHeader},
        reducers::{publish::entry_data_to_entry_aspect_data, send},
        state::NetworkState,
    },
    state::State,
};
use holochain_core_types::{chain_header::ChainHeader, error::HolochainError};
use lib3h_protocol::{
    data_types::{EntryData, ProvidedEntryData},
    protocol_client::Lib3hClientProtocol,
};

use crate::{network::actions::Response, state::StateWrapper};
use holochain_persistence_api::cas::content::{Address, AddressableContent};

/// Send to network a request to publish a header entry alone
/// This is similar to publishing a regular entry but it has its own special dummy header.
fn publish_header(
    network_state: &mut NetworkState,
    root_state: &State,
    chain_header: ChainHeader,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } =
        create_entry_with_header_for_header(&StateWrapper::from(root_state.clone()), chain_header)?;
    send(
        network_state,
        Lib3hClientProtocol::PublishEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap().into(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: entry.address().into(),
                aspect_list: vec![entry_data_to_entry_aspect_data(&EntryAspect::Content(
                    entry,
                    header,
                ))],
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
    publish_header(network_state, root_state, entry_with_header.header)
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
        Response::from(NetworkActionResponse::PublishHeaderEntry(match result {
            Ok(_) => Ok(address.clone()),
            Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
        })),
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
