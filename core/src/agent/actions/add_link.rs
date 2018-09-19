use action::{Action, ActionWrapper};
use agent::state::{ActionResponse, AgentState};
use chain::SourceChain;
use context::Context;
use hash_table::{
    links_entry::{LinkActionKind, LinkEntry},
    sys_entry::ToEntry,
    HashTable,
};
use instance::Observer;
use std::sync::{mpsc::Sender, Arc};

/// Do the AddLink Action against an agent state:
/// 1. Validate Link
/// 2. Commit LinkEntry
/// 3. Add Link metadata in HashTable
pub(crate) fn reduce_add_link(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::AddLink);

    // TODO #277
    // Validate Link Here

    // Create and Commit a LinkEntry on source chain
    let link_entry = LinkEntry::from_link(LinkActionKind::ADD, link);
    let res = state.chain().clone().commit_entry(&link_entry.to_entry());
    let mut response = if res.is_ok() {
        Ok(res.unwrap().entry().clone())
    } else {
        Err(res.err().unwrap())
    };

    // Add Link to HashTable (adds to the LinkListEntry Meta)
    let res = state.chain().table().add_link(link);
    if res.is_err() {
        response = Err(res.err().unwrap());
    }

    // Insert reponse in state
    state.actions().insert(
        action_wrapper.clone(),
        ActionResponse::LinkEntries(response),
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use action::ActionWrapper;
    use agent::{
        actions::get_links::reduce_get_links,
        state::{reduce_commit_entry, tests::test_agent_state},
    };
    use error::HolochainError;
    use hash_table::{
        entry::Entry,
        links_entry::{tests::create_test_link, Link},
    };
    use instance::tests::{test_context, test_instance_blank};
    use key::Key;
    use nucleus::ribosome::api::get_links::GetLinksArgs;

    /// test for reducing AddLink
    #[test]
    fn test_reduce_add_link_empty() {
        let mut state = test_agent_state();

        let link = create_test_link();
        let action_wrapper = ActionWrapper::new(Action::AddLink(link));

        let instance = test_instance_blank();

        reduce_add_link(
            test_context("camille"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            Some(&ActionResponse::LinkEntries(Err(
                HolochainError::ErrorGeneric("Entry from base not found".to_string())
            ))),
            state.actions().get(&action_wrapper),
        );
    }

    /// test for reducing AddLink
    #[test]
    fn test_reduce_add_link() {
        let context = test_context("camille");

        let e1 = Entry::new("app1", "alex");
        let e2 = Entry::new("app1", "billy");

        let t1 = "child".to_string();

        let req1 = GetLinksArgs {
            entry_hash: e1.key(),
            tag: t1.clone(),
        };

        let link = Link::new(&e1.key(), &e2.key(), &t1);

        let action_commit_e1 = ActionWrapper::new(Action::Commit(e1.clone()));
        let action_commit_e2 = ActionWrapper::new(Action::Commit(e2.clone()));
        let action_lap = ActionWrapper::new(Action::AddLink(link));
        let action_gl = ActionWrapper::new(Action::GetLinks(req1));

        let mut state = test_agent_state();

        let instance = test_instance_blank();

        reduce_commit_entry(
            context.clone(),
            &mut state,
            &action_commit_e1,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );
        reduce_commit_entry(
            context.clone(),
            &mut state,
            &action_commit_e2,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );
        reduce_add_link(
            context.clone(),
            &mut state,
            &action_lap,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );
        reduce_get_links(
            context.clone(),
            &mut state,
            &action_gl,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        let mut res = Vec::new();
        res.push(e2.key());

        assert_eq!(
            Some(&ActionResponse::GetLinks(Ok(res))),
            state.actions().get(&action_gl),
        );
    }
}
