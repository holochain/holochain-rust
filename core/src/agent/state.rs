use action::{Action, ActionWrapper, AgentReduceFn};
use agent::keys::Keys;
use chain::{Chain, SourceChain};
use context::Context;
use error::HolochainError;
use hash_table::{
    HashString, HashTable,
    links_entry::LinkEntry, links_entry::LinkActionKind,
    sys_entry::ToEntry,
};
use chain::pair::Pair;
use instance::Observer;
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc},
};


#[derive(Clone, Debug, PartialEq)]
/// struct to track the internal state of an agent exposed to reducers/observers
pub struct AgentState {
    keys: Option<Keys>,
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    actions: HashMap<ActionWrapper, ActionResponse>,
    chain: Chain,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new(chain: &Chain) -> AgentState {
        AgentState {
            keys: None,
            actions: HashMap::new(),
            chain: chain.clone(),
        }
    }

    /// getter for a copy of self.keys
    pub fn keys(&self) -> Option<Keys> {
        self.keys.clone()
    }

    /// getter for the chain
    pub fn chain(&self) -> &Chain {
        &self.chain
    }

    /// getter for a copy of self.actions
    /// uniquely maps action executions to the result of the action
    pub fn actions(&self) -> HashMap<ActionWrapper, ActionResponse> {
        self.actions.clone()
    }

    // /// chain getter
    // pub fn chain(&self) -> Option<Chain<MemTable>> { self.chain.clone() }
}

#[derive(Clone, Debug, PartialEq)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
pub enum ActionResponse {
    CommitEntry(Result<Pair, HolochainError>),
    GetEntry(Option<Pair>),
    GetLinks(Result<Vec<HashString>, HolochainError>),
    LinkAppEntries(Result<Pair, HolochainError>),
}

// @TODO abstract this to a standard trait
// @see https://github.com/holochain/holochain-rust/issues/196
impl ActionResponse {
    /// serialize data or error to JSON
    // @TODO implement this as a round tripping trait
    // @see https://github.com/holochain/holochain-rust/issues/193
    pub fn to_json(&self) -> String {
        match self {
            ActionResponse::CommitEntry(result) => match result {
                Ok(pair) => format!("{{\"hash\":\"{}\"}}", pair.entry().key()),
                Err(err) => (*err).to_json(),
            },
            ActionResponse::GetEntry(result) => match result {
                Some(pair) => pair.to_json(),
                None => "".to_string(),
            },
            ActionResponse::GetLinks(result) => match result {
                Ok(hash_list) =>  {
                    json!(hash_list).as_str().expect("should jsonify").to_string()
                },
                Err(err) => (*err).to_json(),
            },
            ActionResponse::LinkAppEntries(result) => match result {
                Ok(pair) => format!("{{\"hash\":\"{}\"}}", pair.entry().key()),
                Err(err) => (*err).to_json(),
            },
        }
    }
}

/// Do the LinkAppEntries Action against an agent state:
/// 1. Validate Link
/// 2. Commit LinkEntry
/// 3. Add Link metadata in HashTable
fn reduce_link_app_entries(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::LinkAppEntries);

    // TODO #277
    // Validate Link Here

    // Create and Commit a LinkEntry on source chain
    let link_entry = LinkEntry::new_from_link(LinkActionKind::ADD, link);
    let mut response =  state.chain.commit_entry(&link_entry.to_entry());

    // Add Link to HashTable (adds to the LinkListEntry Meta)
    let res = state.chain.table().add_link(link);
    if res.is_err() {
        response = Err(res.err().unwrap());
    }

    // Insert reponse in state
    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::LinkAppEntries(response),
    );
}


/// Do the GetLinks Action against an agent state
fn reduce_get_links(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let links_request = unwrap_to!(action => Action::GetLinks);

//    // Look for entry's link metadata
    let res = state.chain.table().links(links_request);
    if res.is_err() {
        state
        .actions
        .insert(
            action_wrapper.clone(),
        ActionResponse::GetLinks(Err(res.err().unwrap())));
        return;
    }
    let maybe_lle = res.unwrap();
    if maybe_lle.is_none() {
        state
            .actions
            .insert(
                action_wrapper.clone(),
                ActionResponse::GetLinks(Ok(Vec::new())));
        return;
    }
    let lle = maybe_lle.unwrap();

    // Extract list of target hashes
    let mut link_hashes = Vec::new();
    for link in lle.links {
        link_hashes.push(link.target().to_string());
    }

    // Insert reponse in state
    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::GetLinks(Ok(link_hashes.clone())));
}


/// do a commit action against an agent state
/// intended for use inside the reducer, isolated for unit testing
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
/// @TODO is there a way to reduce that doesn't block indefinitely on callback fns?
/// @see https://github.com/holochain/holochain-rust/issues/222
fn reduce_commit_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::CommitEntry);

    // @TODO validation dispatch should go here rather than upstream in invoke_commit
    // @see https://github.com/holochain/holochain-rust/issues/256

    let response = state.chain.commit_entry(&entry);
    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::CommitEntry(response),
    );
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn reduce_get_entry(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => Action::GetEntry);

    let result = state.chain.entry(&key.clone());

    // @TODO if the get fails local, do a network get
    // @see https://github.com/holochain/holochain-rust/issues/167

    state.actions.insert(
        action_wrapper.clone(),
        ActionResponse::GetEntry(
            result
                .clone()
                .expect("should be able to get entry that we just added"),
        ),
    );
}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::CommitEntry(_) => Some(reduce_commit_entry),
        Action::GetEntry(_) => Some(reduce_get_entry),
        Action::GetLinks(_) => Some(reduce_get_links),
        Action::LinkAppEntries(_) => Some(reduce_link_app_entries),
        _ => None,
    }
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    context: Arc<Context>,
    old_state: Arc<AgentState>,
    action_wrapper: &ActionWrapper,
    action_channel: &Sender<ActionWrapper>,
    observer_channel: &Sender<Observer>,
) -> Arc<AgentState> {
    let handler = resolve_reducer(action_wrapper);
    match handler {
        Some(f) => {
            let mut new_state: AgentState = (*old_state).clone();
            f(
                context,
                &mut new_state,
                &action_wrapper,
                action_channel,
                observer_channel,
            );
            Arc::new(new_state)
        }
        None => old_state,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use chain::tests::test_chain;
    use error::HolochainError;
    use hash_table::{
        entry::Entry,
        links_entry::Link,
    };
    use instance::tests::{test_context, test_instance_blank};
    use std::{collections::HashMap, sync::Arc};
    use action::ActionWrapper;
    use nucleus::ribosome::api::get_links::GetLinksArgs;
    use chain::pair::tests::test_pair;

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new(&test_chain())
    }

    /// dummy action response for a successful commit as test_pair()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::CommitEntry(Ok(test_pair()))
    }

    /// dummy action response for a successful get as test_pair()
    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::GetEntry(Some(test_pair()))
    }

    #[test]
    /// smoke test for building a new AgentState
    fn agent_state_new() {
        test_agent_state();
    }

    #[test]
    /// test for the agent state keys getter
    fn agent_state_keys() {
        assert_eq!(None, test_agent_state().keys());
    }

    #[test]
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state().actions());
    }

    /// test for reducing GetLinks
    #[test]
    fn test_reduce_get_links_empty() {
        let mut state = test_agent_state();

        let req1 = GetLinksArgs{entry_hash: "0x42".to_string(), tag: "child".to_string()};
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

    /// test for reducing LinkAppEntries
    #[test]
    fn test_reduce_link_app_entries_empty() {
        let mut state = test_agent_state();

        let link = Link::new("0x12", "0x34", "child");
        let action_wrapper = ActionWrapper::new(Action::LinkAppEntries(link));

        let instance = test_instance_blank();

        reduce_link_app_entries(
            test_context("camille"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            Some(&ActionResponse::LinkAppEntries(Err(HolochainError::ErrorGeneric("Entry from base not found".to_string())))),
            state.actions().get(&action_wrapper),
        );
    }

    /// test for reducing LinkAppEntries
    #[test]
    fn test_reduce_link_app_entries() {
        let context = test_context("camille");

        let e1 = Entry::new("app1", "alex");
        let e2 = Entry::new("app1", "billy");

        let t1 = "child".to_string();

        let req1 = GetLinksArgs{entry_hash:e1.key(), tag: t1.clone()};

        let link = Link::new(&e1.key(), &e2.key(), &t1);

        let action_commit_e1 =  ActionWrapper::new(Action::CommitEntry(e1.clone()));
        let action_commit_e2 =  ActionWrapper::new(Action::CommitEntry(e2.clone()));
        let action_lap = ActionWrapper::new(Action::LinkAppEntries(link));
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
        reduce_link_app_entries(
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

    #[test]
    /// test for reducing commit entry
    fn test_reduce_commit_entry() {
        let mut state = test_agent_state();
        let action_wrapper = test_action_wrapper_commit();

        let instance = test_instance_blank();

        reduce_commit_entry(
            test_context("bob"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            state.actions().get(&action_wrapper),
            Some(&test_action_response_commit()),
        );
    }

    #[test]
    /// test for reducing get entry
    fn test_reduce_get_entry() {
        let mut state = test_agent_state();
        let context = test_context("foo");

        let instance = test_instance_blank();

        let aw1 = test_action_wrapper_get();
        reduce_get_entry(
            Arc::clone(&context),
            &mut state,
            &aw1,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        // nothing has been committed so the get must be None
        assert_eq!(state.actions().get(&aw1), Some(&ActionResponse::GetEntry(None)),);

        // do a round trip
        reduce_commit_entry(
            Arc::clone(&context),
            &mut state,
            &test_action_wrapper_commit(),
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        let aw2 = test_action_wrapper_get();
        reduce_get_entry(
            Arc::clone(&context),
            &mut state,
            &aw2,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(state.actions().get(&aw2), Some(&test_action_response_get()),);
    }

    #[test]
    /// test response to json
    fn test_response_to_json() {
        assert_eq!(
            "{\"hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\"}",
            ActionResponse::CommitEntry(Ok(test_pair())).to_json(),
        );
        assert_eq!(
            "{\"error\":\"some error\"}",
            ActionResponse::CommitEntry(Err(HolochainError::new("some error"))).to_json(),
        );

        assert_eq!(
            "{\"header\":{\"entry_type\":\"testEntryType\",\"timestamp\":\"\",\"link\":null,\"entry_hash\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"entry_signature\":\"\",\"link_same_type\":null},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}",
            ActionResponse::GetEntry(Some(test_pair())).to_json(),
        );
        assert_eq!("", ActionResponse::GetEntry(None).to_json());
    }
}
