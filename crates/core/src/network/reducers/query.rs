use crate::{
    action::{ActionWrapper, QueryKey},
    network::{query::NetworkQuery, reducers::send, state::NetworkState},
    state::State,
};
use holochain_core_types::error::HolochainError;
use holochain_json_api::json::JsonString;
use lib3h_protocol::{data_types::QueryEntryData, protocol_client::Lib3hClientProtocol};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
fn reduce_query_inner(
    network_state: &mut NetworkState,
    key: QueryKey,
    network_query: NetworkQuery,
) -> Result<(), HolochainError> {
    network_state.initialized()?;
    let query_json: JsonString = network_query.into();
    let key_address = match key {
        QueryKey::Entry(key) => (key.id.clone(), key.address),
        QueryKey::Links(key) => (key.id.clone(), key.base_address),
    };
    send(
        network_state,
        Lib3hClientProtocol::QueryEntry(QueryEntryData {
            requester_agent_id: network_state.agent_id.clone().unwrap().into(),
            request_id: key_address.0,
            space_address: network_state.dna_address.clone().unwrap().into(),
            entry_address: key_address.1.into(),
            query: query_json.to_string().into_bytes().into(),
        }),
    )
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_query(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (key_type, payload, maybe_timeout) = unwrap_to!(action => crate::action::Action::Query);
    let network_query = match key_type.clone() {
        QueryKey::Entry(_) => NetworkQuery::GetEntry,
        QueryKey::Links(key) => {
            let (crud_status, query) = unwrap_to!(payload => crate::action::QueryPayload::Links);
            NetworkQuery::GetLinks(key.link_type.clone(), key.tag, *crud_status, query.clone())
        }
    };

    let result = reduce_query_inner(network_state, key_type.clone(), network_query)
        .map(|_| None)
        .unwrap_or_else(|e| Some(Err(e)));
    network_state
        .get_query_results
        .insert(key_type.clone(), result);

    if let Some(timeout) = maybe_timeout {
        network_state
            .query_timeouts
            .insert(key_type.clone(), timeout.clone());
    }
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn reduce_query_timeout(
    network_state: &mut NetworkState,
    _root_state: &State,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => crate::action::Action::QueryTimeout);

    network_state.query_timeouts.remove(key);

    if network_state.get_query_results.get(&key).is_none() {
        return;
    }

    if network_state.get_query_results.get(key).unwrap().is_none() {
        network_state
            .get_query_results
            .insert(key.clone(), Some(Err(HolochainError::Timeout)));
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper, GetEntryKey, GetLinksKey, QueryKey, QueryPayload},
        instance::tests::test_context,
        network::query::{GetLinksNetworkQuery, GetLinksQueryConfiguration},
        state::test_store,
    };
    use holochain_persistence_api::cas::content::AddressableContent;

    use holochain_core_types::{entry::test_entry, error::HolochainError};

    #[test]
    pub fn reduce_get_entry_without_network_initialized() {
        let netname = Some("reduce_get_entry_without_network_initialized");
        let context = test_context("alice", netname);
        let store = test_store(context.clone());

        let entry = test_entry();
        let key = GetEntryKey {
            address: entry.address(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action = Action::Query((QueryKey::Entry(key.clone()), QueryPayload::Entry, None));
        let action_wrapper = ActionWrapper::new(action);

        let store = store.reduce(action_wrapper);
        let maybe_get_entry_result = store
            .network()
            .get_query_results
            .get(&QueryKey::Entry(key.clone()))
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_entry_result,
            Some(Some(Err(HolochainError::ErrorGeneric(
                "Network not initialized".to_string()
            ))))
        );
    }

    #[test]
    // This test needs to be refactored.
    // It is non-deterministically failing with "sending on a closed channel" originating form
    // within the in-memory network.
    #[cfg(feature = "broken-tests")]
    pub fn reduce_get_entry_test() {
        let netname = Some("reduce_get_entry_test");
        let context = test_context("alice", netname);
        let store = test_store(context.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_memory_network_config(netname),
            dna_address: "abcd".into(),
            agent_id: String::from("abcd"),
        }));
        let store = store.reduce(context.clone(), action_wrapper);

        let entry = test_entry();
        let key = GetEntryKey {
            address: entry.address(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::Query((
            QueryKey::Entry(key.clone()),
            QueryPayload::Entry,
        )));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store
            .network()
            .get_entry_with_meta_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(None));
    }

    #[test]
    // This test needs to be refactored.
    // It is non-deterministically failing with "sending on a closed channel" originating form
    // within the in-memory network.
    #[cfg(feature = "broken-tests")]
    pub fn reduce_get_entry_timeout_test() {
        let netname = Some("reduce_get_entry_timeout_test");
        let mut context = test_context("alice", netname);
        let store = test_store(context.clone());
        let store = Arc::new(RwLock::new(store));

        Arc::get_mut(&mut context).unwrap().set_state(store.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_memory_network_config(netname),
            dna_address: "reduce_get_entry_timeout_test".into(),
            agent_id: AgentId::generate_fake("timeout").address().to_string(),
        }));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

        let entry = test_entry();
        let key = GetEntryKey {
            address: entry.address(),
            id: "req_alice_1".to_string(),
        };
        let key = QueryKey::Entry(key.clone());
        let action = Action::Query((key, QueryPayload::Entry));
        let action_wrapper = ActionWrapper::new(action);

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_get_entry_result = store
            .read()
            .unwrap()
            .network()
            .get_query_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(None));

        let action_wrapper = ActionWrapper::new(Action::QueryEntryTimeout(key.clone()));
        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_get_entry_result = store
            .read()
            .unwrap()
            .network()
            .get_entry_with_meta_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_entry_result,
            Some(Some(Err(HolochainError::Timeout)))
        );

        // test that an existing result does not get overwritten by timeout signal
        let entry_with_meta = EntryWithMeta {
            entry: entry.clone(),
            crud_status: CrudStatus::Live,
            maybe_link_update_delete: None,
        };
        let new_key = GetEntryKey {
            address: entry.address(),
            id: "req_alice_2".to_string(),
        };
        let dht_data = QueryEntryResultData {
            msg_id: new_key.id.clone(),
            address: new_key.address.to_string(),
            content: &serde_json::to_value(&Some(entry_with_meta.clone()).unwrap()).unwrap(),
            ..Default::default()
        };

        let action_wrapper = ActionWrapper::new(Action::HandleQueryResult(dht_data));
        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_entry_with_meta_result = store
            .read()
            .unwrap()
            .network()
            .get_entry_with_meta_results
            .get(&new_key)
            .map(|result| result.clone());
        assert!(maybe_entry_with_meta_result.is_some());
        let maybe_entry_with_meta = maybe_entry_with_meta_result.unwrap().unwrap();
        let result = maybe_entry_with_meta.unwrap();
        let entry_with_meta = result.unwrap();
        assert_eq!(entry_with_meta.entry, entry.clone());

        // Ok we got a positive result in the state
        let action_wrapper = ActionWrapper::new(Action::QueryEntryTimeout(new_key.clone()));
        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }
        let maybe_entry_with_meta_result = store
            .read()
            .unwrap()
            .network()
            .get_entry_with_meta_results
            .get(&new_key)
            .map(|result| result.clone());
        // The timeout should not have overwritten the entry
        assert!(maybe_entry_with_meta_result.is_some());
        let maybe_entry_with_meta = maybe_entry_with_meta_result.unwrap().unwrap();
        let entry_with_meta = maybe_entry_with_meta.unwrap().unwrap();
        assert_eq!(entry_with_meta.entry, entry);
    }

    #[test]
    pub fn reduce_get_links_without_network_initialized() {
        let context = test_context("alice", None);
        let store = test_store(context.clone());

        let entry = test_entry();
        let link_type = String::from("test-link");
        let key = GetLinksKey {
            base_address: entry.address(),
            link_type,
            tag: "link-tag".to_string(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let config = GetLinksQueryConfiguration::default();
        let get_links_network_query = GetLinksNetworkQuery::Links(config);
        let payload = QueryPayload::Links((None, get_links_network_query));
        let action = Action::Query((QueryKey::Links(key.clone()), payload, None));
        let action_wrapper = ActionWrapper::new(action);

        let store = store.reduce(action_wrapper);
        let maybe_get_links_result = store
            .network()
            .get_query_results
            .get(&QueryKey::Links(key))
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_links_result,
            Some(Some(Err(HolochainError::ErrorGeneric(
                "Network not initialized".to_string()
            ))))
        );
    }

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
        let action_wrapper = ActionWrapper::new(Action::QueryLinks(key.clone()));

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
        let action_wrapper = ActionWrapper::new(Action::QueryLinks(key.clone()));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

        let maybe_get_entry_result = store
            .read()
            .unwrap()
            .network()
            .get_links_results
            .get(&QueryKey::Links(key))
            .cloned();

        assert_eq!(maybe_get_entry_result, Some(None));

        let action_wrapper = ActionWrapper::new(Action::QueryLinksTimeout(key.clone()));
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
