use action::{Action, ActionWrapper};
use agent::state::{ActionResponse, AgentState};
use context::Context;
use hash_table::HashTable;
use instance::Observer;
use std::sync::{mpsc::Sender, Arc};

/// Do the GetLinks Action against an agent state
pub(crate) fn reduce_get_links(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let links_request = unwrap_to!(action => Action::GetLinks);

    //    // Look for entry's link metadata
    let res = state.chain().table().get_links(links_request);
    if res.is_err() {
        state.actions().insert(
            action_wrapper.clone(),
            ActionResponse::GetLinks(Err(res.err().unwrap())),
        );
        return;
    }
    let maybe_lle = res.unwrap();
    if maybe_lle.is_none() {
        state.actions().insert(
            action_wrapper.clone(),
            ActionResponse::GetLinks(Ok(Vec::new())),
        );
        return;
    }
    let lle = maybe_lle.unwrap();

    // Extract list of target hashes
    let mut link_hashes = Vec::new();
    for link in lle.links {
        link_hashes.push(link.target().clone());
    }

    // Insert reponse in state
    state.actions().insert(
        action_wrapper.clone(),
        ActionResponse::GetLinks(Ok(link_hashes.clone())),
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use action::ActionWrapper;
    use agent::state::tests::test_agent_state;
    use hash::HashString;
    use instance::tests::{test_context, test_instance_blank};
    use nucleus::ribosome::api::get_links::GetLinksArgs;

    /// test for reducing GetLinks
    #[test]
    fn test_reduce_get_links_empty() {
        let mut state = test_agent_state();

        let req1 = GetLinksArgs {
            entry_hash: HashString::from("0x42".to_string()),
            tag: "child".to_string(),
        };
        let action_wrapper = ActionWrapper::new(Action::GetLinks(req1));

        let instance = test_instance_blank();

        reduce_get_links(
            test_context("camille"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            Some(&ActionResponse::GetLinks(Ok(Vec::new()))),
            state.actions().get(&action_wrapper),
        );
    }
}
