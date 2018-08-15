use action::{Action, ActionWrapper, AgentReduceFn};
use agent::keys::Keys;
use chain::Chain;
use hash_table::{entry::Entry, pair::Pair};
use instance::Observer;
use riker::actors::*;
use snowflake;
use state;
use context::Context;
use error::HolochainError;
use hash_table::{entry::Entry, memory::MemTable, pair::Pair};
use instance::Observer;
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc},
};
use chain::ChainProtocol;
use hash_table::actor::HashTableActor;
use hash_table::HashTable;
use futures::executor::block_on;
use riker_patterns::ask::ask;
use chain::CHAIN_SYS;
use hash_table::actor::HASH_TABLE_SYS;

#[derive(Clone, Debug, PartialEq)]
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
    chain: ActorRef<ChainProtocol>,
}

impl AgentState {
    /// builds a new, empty AgentState
    pub fn new<HT: HashTable>(table: HT) -> AgentState {

        let table_props = HashTableActor::props(table);
        let table_actor = HASH_TABLE_SYS.actor_of(table_props, "table").unwrap();

        let chain_props = Chain::props(table_actor.clone());
        let chain = CHAIN_SYS.actor_of(chain_props, "chain").unwrap();

        AgentState {
            keys: None,
            top_pair: None,
            actions: HashMap::new(),
            chain,
        }
    }

    fn ask_chain_for_response(&self, message: ChainProtocol) -> ChainProtocol {
        let a = ask(
            &(*CHAIN_SYS),
            &self.chain,
            message
        );
        block_on(a).unwrap()
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
}

#[derive(Clone, Debug, PartialEq)]
/// the agent's response to an action
/// stored alongside the action in AgentState::actions to provide a state history that observers
/// poll and retrieve
pub enum ActionResponse {
    Commit(Result<Pair, HolochainError>),
    Get(Option<Pair>),
}

// @TODO abstract this to a standard trait
// @see https://github.com/holochain/holochain-rust/issues/196
impl ActionResponse {
    /// serialize data or error to JSON
    // @TODO implement this as a round tripping trait
    // @see https://github.com/holochain/holochain-rust/issues/193
    pub fn to_json(&self) -> String {
        match self {
            ActionResponse::Commit(result) => match result {
                Ok(pair) => format!("{{\"hash\":\"{}\"}}", pair.entry().key()),
                Err(err) => (*err).to_json(),
            },
            ActionResponse::Get(result) => match result {
                Some(pair) => pair.to_json(),
                None => "".to_string(),
            },
        }
    }
}

/// do a commit action against an agent state
/// intended for use inside the reducer, isolated for unit testing
/// callback checks (e.g. validate_commit) happen elsewhere because callback functions cause
/// action reduction to hang
/// @TODO is there a way to reduce that doesn't block indefinitely on callback fns?
/// @see https://github.com/holochain/holochain-rust/issues/222
fn reduce_commit(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let entry = unwrap_to!(action => Action::Commit);

    // @TODO successfully validate before pushing a commit
    // @see https://github.com/holochain/holochain-rust/issues/97

    let response = state.ask_chain_for_response(
        ChainProtocol::Push(entry.clone()),
    );
    let result = unwrap_to!(response => ChainProtocol::PushResult);
    // commit returns the entry key not the pair, from the action's perspective as this is
    // what the zome API expects
    let result = result.clone().unwrap().entry().key();

    state
        .actions
        .insert(
            action_wrapper.clone(),
            ActionResult::Commit(result),
        );
}

/// do a get action against an agent state
/// intended for use inside the reducer, isolated for unit testing
fn reduce_get(
    _context: Arc<Context>,
    state: &mut AgentState,
    action_wrapper: &ActionWrapper,
    _action_channel: &Sender<ActionWrapper>,
    _observer_channel: &Sender<Observer>,
) {
    let action = action_wrapper.action();
    let key = unwrap_to!(action => Action::Get);

    let response = state.ask_chain_for_response(
        ChainProtocol::GetEntry(key.clone()),
    );
    let result = unwrap_to!(response => ChainProtocol::GetEntryResult);

    // @TODO if the get fails local, do a network get
    // @see https://github.com/holochain/holochain-rust/issues/167

    state
        .actions
        .insert(
            action_wrapper.clone(),
            ActionResult::Get(result.clone().unwrap()),
        );
}

/// maps incoming action to the correct handler
fn resolve_reducer(action_wrapper: &ActionWrapper) -> Option<AgentReduceFn> {
    match action_wrapper.action() {
        Action::Commit(_) => Some(reduce_commit),
        Action::Get(_) => Some(reduce_get),
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
    use super::{reduce_commit, reduce_get, ActionResponse, AgentState};
    use action::tests::{test_action_wrapper_commit, test_action_wrapper_get};
    use error::HolochainError;
    use hash_table::pair::tests::test_pair;
    use instance::tests::{test_context, test_instance_blank};
    use std::collections::HashMap;
    use riker::actor::Tell;
    use std::thread;

    #[test]
    fn test_actor_receive() {
        let state = test_agent_state();

        state.chain.tell("hi".to_string(), None);
        let chain = state.chain.clone();
        let handle = thread::spawn(move || {
            chain.tell("thread hi!".to_string(), None);
        });
        handle.join().unwrap();
    }

    /// dummy agent state
    pub fn test_agent_state() -> AgentState {
        AgentState::new()
    }

    /// dummy action response for a successful commit as test_pair()
    pub fn test_action_response_commit() -> ActionResponse {
        ActionResponse::Commit(Ok(test_pair()))
    }

    /// dummy action response for a successful get as test_pair()
    pub fn test_action_response_get() -> ActionResponse {
        ActionResponse::Get(Some(test_pair()))
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

        reduce_commit(
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

        reduce_get(
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
            ActionResponse::Commit(Ok(test_pair())).to_json(),
        );
        assert_eq!(
            "{\"error\":\"some error\"}",
            ActionResponse::Commit(Err(HolochainError::new("some error"))).to_json(),
        );

        assert_eq!(
            "{\"header\":{\"entry_type\":\"testEntryType\",\"time\":\"\",\"next\":null,\"entry\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"type_next\":null,\"signature\":\"\"},\"entry\":{\"content\":\"test entry content\",\"entry_type\":\"testEntryType\"}}",
            ActionResponse::Get(Some(test_pair())).to_json(),
        );
        assert_eq!("", ActionResponse::Get(None).to_json(),);
    }
}
