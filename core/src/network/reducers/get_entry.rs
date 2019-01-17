use crate::{
    action::{ActionWrapper, GetEntryKey},
    context::Context,
    network::{reducers::send, state::NetworkState},
};
use holochain_core_types::error::HolochainError;
use holochain_net_connection::protocol_wrapper::{GetDhtData, ProtocolWrapper};
use std::sync::Arc;

fn inner(network_state: &mut NetworkState, key: &GetEntryKey) -> Result<(), HolochainError> {
    network_state.initialized()?;

    send(
        network_state,
        ProtocolWrapper::GetDht(GetDhtData {
            msg_id: key.id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            from_agent_id: network_state.agent_id.clone().unwrap(),
            address: key.address.to_string(),
        }),
    )
}

pub fn reduce_get_entry(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => crate::action::Action::GetEntry);

    let result = match inner(network_state, &key) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_entry_with_meta_results
        .insert(key.clone(), result);
}

pub fn reduce_get_entry_timeout(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => crate::action::Action::GetEntryTimeout);

    if network_state.get_entry_with_meta_results.get(key).is_none() {
        return;
    }

    if network_state
        .get_entry_with_meta_results
        .get(key)
        .unwrap()
        .is_none()
    {
        network_state
            .get_entry_with_meta_results
            .insert(key.clone(), Some(Err(HolochainError::Timeout)));
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper, GetEntryKey, NetworkSettings},
        context::test_mock_config,
        instance::tests::test_context,
        state::test_store,
    };
    use holochain_core_types::{
        agent::AgentId, crud_status::CrudStatus, entry::EntryWithMeta, error::HolochainError,
    };
    use holochain_net_connection::protocol_wrapper::DhtData;
    use std::sync::{Arc, RwLock};

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
        let action_wrapper = ActionWrapper::new(Action::GetEntry(key.clone()));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store
            .network()
            .get_entry_with_meta_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_entry_result,
            Some(Some(Err(HolochainError::ErrorGeneric(
                "Network not initialized".to_string()
            ))))
        );
    }

    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};

    #[test]
    pub fn reduce_get_entry_test() {
        let netname = Some("reduce_get_entry_test");
        let context = test_context("alice", netname);
        let store = test_store(context.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_mock_config(netname),
            dna_address: "abcd".into(),
            agent_id: String::from("abcd"),
        }));
        let store = store.reduce(context.clone(), action_wrapper);

        let entry = test_entry();
        let key = GetEntryKey {
            address: entry.address(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::GetEntry(key.clone()));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store
            .network()
            .get_entry_with_meta_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(None));
    }

    #[test]
    pub fn reduce_get_entry_timeout_test() {
        let netname = Some("reduce_get_entry_timeout_test");
        let mut context = test_context("alice", netname);
        let store = test_store(context.clone());
        let store = Arc::new(RwLock::new(store));

        Arc::get_mut(&mut context).unwrap().set_state(store.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: test_mock_config(netname),
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
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::GetEntry(key.clone()));

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
        assert_eq!(maybe_get_entry_result, Some(None));

        let action_wrapper = ActionWrapper::new(Action::GetEntryTimeout(key.clone()));
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
            maybe_crud_link: None,
        };
        let new_key = GetEntryKey {
            address: entry.address(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        };
        let dht_data = DhtData {
            msg_id: new_key.id.clone(),
            address: new_key.address.to_string(),
            content: serde_json::from_str(
                &serde_json::to_string(&Some(entry_with_meta.clone())).unwrap(),
            )
            .unwrap(),
            ..Default::default()
        };

        let action_wrapper = ActionWrapper::new(Action::HandleGetResult(dht_data));
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
        println!("{:?}", result);
        let entry_with_meta = result.unwrap();
        assert_eq!(entry_with_meta.entry, entry.clone());

        // Ok we got a positive result in the state
        let action_wrapper = ActionWrapper::new(Action::GetEntryTimeout(new_key.clone()));
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
}
