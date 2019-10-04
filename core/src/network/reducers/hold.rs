use crate::{
    action::ActionWrapper,
    network::{/*entry_aspect::EntryAspect, reducers::send,*/ state::NetworkState},
    state::State,
};

/*use lib3h_protocol::{
    data_types::{EntryData, ProvidedEntryData},
    protocol_client::Lib3hClientProtocol,
};

use holochain_persistence_api::cas::content::AddressableContent;
*/
pub fn reduce_hold(
    _network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let _entry_with_header = unwrap_to!(action => crate::action::Action::Hold);

    panic!("HoldEntry deleted");
/*
    // TODO: use this result as a network action
    let _result = send(
        network_state,
        Lib3hClientProtocol::HoldEntry(ProvidedEntryData {
            space_address: network_state.dna_address.clone().unwrap(),
            provider_agent_id: network_state.agent_id.clone().unwrap().into(),
            entry: EntryData {
                entry_address: entry_with_header.entry.address().clone(),
                aspect_list: vec![EntryAspect::Content(
                    entry_with_header.entry.clone(),
                    entry_with_header.header.clone(),
                )
                .into()],
            },
        }),
    );
*/
    // network_state.actions.insert(
    //     action_wrapper.clone(),
    //     ActionResponse::Hold(match result {
    //         Ok(_) => Ok(address.clone()),
    //         Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
    //     }),
    // );
}
