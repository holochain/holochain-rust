use action::{Action, ActionWrapper, AgentReduceFn};
use agent::keys::Keys;
use chain::Chain;
use context::Context;
use error::HolochainError;
use hash_table::{
    HashString, HashTable, pair_meta::PairMeta,
    entry::Entry, memory::MemTable, pair::Pair,
    links_entry::LinkEntry, links_entry::LinkActionKind, links_entry::LinkListEntry,
    sys_entry::ToEntry,
};
use instance::Observer;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc},
};
use std::str::FromStr;
// #[macro_use]
use serde_json;

#[derive(Clone, Debug, PartialEq, Default)]
/// struct to track the internal state of an agent exposed to reducers/observers
pub struct AgentState {
    keys: Option<Keys>,
    // @TODO how should this work with chains/HTs?
    // @see https://github.com/holochain/holochain-rust/issues/137
    // @see https://github.com/holochain/holochain-rust/issues/135
    top_pair: Option<Pair>,
    /// every action and the result of that action
    // @TODO this will blow up memory, implement as some kind of dropping/FIFO with a limit?
    // @see https://github.com/holochain/holochain-rust/issues/166
    actions: HashMap<ActionWrapper, ActionResponse>,

    // /// Hold the agent's source chain as a hash table store in memory
    // /// FIXME stateful stuff should be in instance??
    // chain: Option<Chain<MemTable>>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new() -> AgentState {
        AgentState {
            keys: None,
            top_pair: None,
            actions: HashMap::new(),
            // chain: Some(Chain::new(Rc::new(MemTable::new()))),
        }
    }

    /// getter for a copy of self.keys
    pub fn keys(&self) -> Option<Keys> {
        self.keys.clone()
    }

    /// getter for a copy of self.top_pair
    /// should be used with a source chain for validation/safety
    pub fn top_pair(&self) -> Option<Pair> {
        self.top_pair.clone()
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
            // FIXME copy of ActionResponse::CommitEntry(result) , should merge with match
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
/// 3. TODO do something on the DHT?
fn reduce_link_app_entries(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let link = unwrap_to!(action => Action::LinkAppEntries);

    // Validate Link
    // FIXME

    // Create and Commit a LinkEntry on source chain
    let link_entry = LinkEntry::new_from_link(LinkActionKind::ADD, link);
    let response = Err(HolochainError::LoggingError); // state.chain().push_entry(&link_entry.to_entry());

    // Add LinkListEntry to HashTable
    // FIXME: Create&Commit or Update in HashTable a LinkListEntry with key = base-entry-hash + tag
    // FIXME: Create/Update metadata for base entry

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

    // Look for entry's metadata
    let result : Result<Option<PairMeta>, HolochainError> = Err(HolochainError::LoggingError);
    // let result = state.chain().table().get_meta(links_request.key());
    if result.is_err() || result.clone().unwrap().is_none() {
        state
            .actions
            .insert(action_wrapper.clone(),
                    ActionResponse::GetLinks(Err(HolochainError::ErrorGeneric("base entry not found".to_string()))));
        return;
    }
    let result = result.unwrap().unwrap();

    // Get LinkListEntry in HashTable
    let links_pair: Result<Option<Pair>, HolochainError> = Err(HolochainError::LoggingError);
    // let links_pair = state.chain().table().get(&result.value());
    if links_pair.is_err() || links_pair.clone().unwrap().is_none() {
        state
            .actions
            .insert(action_wrapper.clone(),
                    ActionResponse::GetLinks(Err(HolochainError::ErrorGeneric("links entry not found".to_string()))));
        return;
    }
    let links_pair = links_pair.unwrap().unwrap();

    // Extract list of target hashes
    let links_entry : LinkListEntry = serde_json::from_str(&links_pair.entry().content()).expect("entry is not a valid LinkListEntry");
    let mut link_hashes = Vec::new();
    for link in links_entry.links {
        link_hashes.push(link.target);
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

    // add entry to source chain
    // @TODO this does nothing!
    // it needs to get something stateless from the agent state that points to
    // something stateful that can handle an entire hash table (e.g. actor)
    // @see https://github.com/holochain/holochain-rust/issues/135
    // @see https://github.com/holochain/holochain-rust/issues/148
    let mut chain = Chain::new(Rc::new(MemTable::new()));

    let response = chain.push_entry(&entry);
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

    // get pair from source chain
    // @TODO this does nothing!
    // it needs to get something stateless from the agent state that points to
    // something stateful that can handle an entire hash table (e.g. actor)
    // @see https://github.com/holochain/holochain-rust/issues/135
    // @see https://github.com/holochain/holochain-rust/issues/148

    // drop in a dummy entry for testing
    let mut chain = Chain::new(Rc::new(MemTable::new()));
    let e = Entry::new("testEntryType", "test entry content");
    chain.push_entry(&e).expect("test entry should be valid");

    // @TODO if the get fails local, do a network get
    // @see https://github.com/holochain/holochain-rust/issues/167

    let result = chain
        .entry(&key)
        .expect("should be able to get entry that we just added");
    state
        .actions
        .insert(action_wrapper.clone(), ActionResponse::GetEntry(result.clone()));
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
    use super::{reduce_commit_entry, reduce_get_entry, ActionResponse, AgentState};
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use error::HolochainError;
    use hash_table::pair::tests::test_pair;
    use instance::tests::{test_context, test_instance_blank};
    use std::collections::HashMap;

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new()
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
    /// test for the agent state top pair getter
    fn agent_state_top_pair() {
        assert_eq!(None, test_agent_state().top_pair());
    }

    #[test]
    /// test for the agent state actions getter
    fn agent_state_actions() {
        assert_eq!(HashMap::new(), test_agent_state().actions());
    }

    #[test]
    /// test for reducing commit
    fn test_reduce_commit() {
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
    /// test for reducing get
    fn test_reduce_get() {
        let mut state = test_agent_state();
        let action_wrapper = test_action_wrapper_get();

        let instance = test_instance_blank();

        reduce_get_entry(
            test_context("foo"),
            &mut state,
            &action_wrapper,
            &instance.action_channel().clone(),
            &instance.observer_channel().clone(),
        );

        assert_eq!(
            state.actions().get(&action_wrapper),
            Some(&test_action_response_get()),
        );
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
