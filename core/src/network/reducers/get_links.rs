use crate::{
    action::{ActionWrapper, GetLinksKey},
    network::{
        query::{GetLinksNetworkQuery, NetworkQuery},
        reducers::send,
        state::NetworkState,
    },
    state::State,
};

use holochain_core_types::{crud_status::CrudStatus, error::HolochainError};
use holochain_json_api::json::JsonString;

use lib3h_protocol::{data_types::QueryEntryData, protocol_client::Lib3hClientProtocol};

use holochain_persistence_api::hash::HashString;
use std::convert::TryInto;

fn reduce_get_links_inner(
    network_state: &mut NetworkState,
    key: &GetLinksKey,
    get_links_query: &GetLinksNetworkQuery,
    crud_status: &Option<CrudStatus>,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let query_json: JsonString = NetworkQuery::GetLinks(
        key.link_type.clone(),
        key.tag.clone(),
        crud_status.clone(),
        get_links_query.clone(),
    )
    .into();
    send(
        network_state,
        Lib3hClientProtocol::QueryEntry(QueryEntryData {
            requester_agent_id: network_state.agent_id.clone().unwrap().into(),
            request_id: key.id.clone(),
            // TODO return result these addresses as errors
            space_address: network_state
                .dna_address
                .clone()
                .unwrap()
                .try_into()
                .expect("space address from base58 string"),
            entry_address: HashString::from(key.base_address.clone())
                .try_into()
                .expect("entry adress from base58 string"),
            query: query_json.to_string().into_bytes(),
        }),
    )
}




#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper, GetLinksKey},
        instance::tests::test_context,
        network::query::{GetLinksNetworkQuery,GetLinksQueryConfiguration},
        state::test_store,
    };
    use holochain_core_types::error::HolochainError;
    //use std::sync::{Arc, RwLock};

    #[test]
    pub fn reduce_get_links_without_network_initialized() {
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let link_type = String::from("test-link");
        let key = GetLinksKey {
            base_address: entry.address(),
            link_type: link_type,
            tag: "link-tag".to_string(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let config = GetLinksQueryConfiguration
        {
            headers : false
        };
        let action_wrapper = ActionWrapper::new(Action::GetLinks((
            key.clone(),
            None,
            GetLinksNetworkQuery::Links(config),
        )));

        let store = store.reduce(action_wrapper);
        let maybe_get_links_result = store
            .network()
            .get_links_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_links_result,
            Some(Some(Err(HolochainError::ErrorGeneric(
                "Network not initialized".to_string()
            ))))
        );
    }

    use holochain_core_types::entry::test_entry;
    use holochain_persistence_api::cas::content::AddressableContent;

    #[test]
    // This test needs to be refactored.
    // It is non-deterministically failing with "sending on a closed channel" originating form
    // within the in-memory network.
    #[cfg(feature = "broken-tests")]
    pub fn reduce_get_links_test() {
        let netname = Some("reduce_get_links_test");
        let context = test_context("alice", netname);
        let store = test_store(context.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_memory_network_config(netname),
            dna_address: "reduce_get_links_test".into(),
            agent_id: String::from("alice"),
        }));
        let store = store.reduce(action_wrapper);

        let entry = test_entry();
        let link_type = String::from("test-link");
        let key = GetLinksKey {
            base_address: entry.address(),
            link_type: link_type.clone(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::GetLinks(key.clone()));

        let store = store.reduce(action_wrapper);
        let maybe_get_entry_result = store.network().get_links_results.get(&key).cloned();

        assert_eq!(maybe_get_entry_result, Some(None));
    }

    #[test]
    // This test needs to be refactored.
    // It is non-deterministically failing with "sending on a closed channel" originating form
    // within the in-memory network.
    #[cfg(feature = "broken-tests")]
    pub fn reduce_get_links_timeout_test() {
        let netname = Some("reduce_get_links_timeout_test");
        let mut context = test_context("alice", netname);
        let store = test_store(context.clone());
        let store = Arc::new(RwLock::new(store));

        Arc::get_mut(&mut context).unwrap().set_state(store.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_memory_network_config(netname),
            dna_address: "reduce_get_links_timeout_test".into(),
            agent_id: String::from("alice"),
        }));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

        let entry = test_entry();
        let link_type = String::from("test-link");
        let key = GetLinksKey {
            base_address: entry.address(),
            link_type: link_type.clone(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::GetLinks(key.clone()));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

        let maybe_get_entry_result = store
            .read()
            .unwrap()
            .network()
            .get_links_results
            .get(&key)
            .cloned();

        assert_eq!(maybe_get_entry_result, Some(None));

        let action_wrapper = ActionWrapper::new(Action::GetLinksTimeout(key.clone()));
        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_get_entry_result = store
            .read()
            .unwrap()
            .network()
            .get_links_results
            .get(&key)
            .cloned();

        assert_eq!(
            maybe_get_entry_result,
            Some(Some(Err(HolochainError::Timeout)))
        );
    }
}
