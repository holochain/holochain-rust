use crate::{
    action::ActionWrapper,
    context::Context,
    network::{reducers::send, state::NetworkState},
};
use holochain_core_types::{cas::content::Address, error::HolochainError};
use holochain_net_connection::protocol_wrapper::{GetDhtMetaData, ProtocolWrapper};
use std::sync::Arc;

fn inner(
    network_state: &mut NetworkState,
    address: &Address,
    tag: &String,
    id: &String,
) -> Result<(), HolochainError> {
    network_state.initialized()?;

    send(
        network_state,
        ProtocolWrapper::GetDhtMeta(GetDhtMetaData {
            msg_id: id.clone(),
            dna_address: network_state.dna_address.clone().unwrap(),
            from_agent_id: network_state.agent_id.clone().unwrap(),
            address: address.to_string(),
            attribute: format!("link__{}", tag),
        }),
    )
}

pub fn reduce_get_links(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let (address, tag, id) = unwrap_to!(action => crate::action::Action::GetLinks);

    let result = match inner(network_state, &address, tag, id) {
        Ok(()) => None,
        Err(err) => Some(Err(err)),
    };

    network_state
        .get_links_results
        .insert((address.clone(), tag.clone(), id.clone()), result);
}

pub fn reduce_get_links_timeout(
    _context: Arc<Context>,
    network_state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => crate::action::Action::GetLinksTimeout);

    if network_state.get_links_results.get(key).is_none() {
        return;
    }

    if network_state.get_links_results.get(key).unwrap().is_none() {
        network_state
            .get_links_results
            .insert(key.clone(), Some(Err(HolochainError::Timeout)));
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        action::{Action, ActionWrapper, NetworkSettings},
        context::mock_network_config,
        instance::tests::test_context,
        state::test_store,
    };
    use holochain_core_types::error::HolochainError;
    use std::sync::{Arc, RwLock};

    #[test]
    pub fn reduce_get_links_without_network_initialized() {
        let context = test_context("alice");
        let store = test_store(context.clone());

        let entry = test_entry();
        let tag = String::from("test-tag");
        let key = (entry.address(), tag.clone(), snowflake::ProcessUniqueId::new().to_string());
        let action_wrapper = ActionWrapper::new(Action::GetLinks(key.clone()));

        let store = store.reduce(context.clone(), action_wrapper);
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

    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};

    #[test]
    pub fn reduce_get_links_test() {
        let context = test_context("alice");
        let store = test_store(context.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: mock_network_config(),
            dna_address: "abcd".into(),
            agent_id: String::from("abcd"),
        }));
        let store = store.reduce(context.clone(), action_wrapper);

        let entry = test_entry();
        let tag = String::from("test-tag");
        let key = (entry.address(), tag.clone(), snowflake::ProcessUniqueId::new().to_string());
        let action_wrapper = ActionWrapper::new(Action::GetLinks(key.clone()));

        let store = store.reduce(context.clone(), action_wrapper);
        let maybe_get_entry_result = store
            .network()
            .get_links_results
            .get(&key)
            .map(|result| result.clone());
        assert_eq!(maybe_get_entry_result, Some(None));
    }

    #[test]
    pub fn reduce_get_links_timeout_test() {
        let mut context = test_context("alice");
        let store = test_store(context.clone());
        let store = Arc::new(RwLock::new(store));

        Arc::get_mut(&mut context).unwrap().set_state(store.clone());

        let action_wrapper = ActionWrapper::new(Action::InitNetwork(NetworkSettings {
            config: mock_network_config(),
            dna_address: "abcd".into(),
            agent_id: String::from("abcd"),
        }));

        {
            let mut new_store = store.write().unwrap();
            *new_store = new_store.reduce(context.clone(), action_wrapper);
        }

        let entry = test_entry();
        let tag = String::from("test-tag");
        let key = (entry.address(), tag.clone(), snowflake::ProcessUniqueId::new().to_string());
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
            .map(|result| result.clone());
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
            .map(|result| result.clone());
        assert_eq!(
            maybe_get_entry_result,
            Some(Some(Err(HolochainError::Timeout)))
        );
    }
}
