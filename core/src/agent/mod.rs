pub mod keys;

use agent::keys::Keys;
use hash_table::entry::Entry;
use state;
use std::sync::{mpsc::Sender, Arc};
use hash_table::pair::Pair;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentState {
    keys: Option<Keys>,
    top_pair: Option<Pair>,
}

impl AgentState {
    pub fn new() -> Self {
        AgentState {
            keys: None,
            top_pair: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Commit(Entry),
}

/// Reduce Agent's state according to provided Action
pub fn reduce(
    old_state: Arc<AgentState>,
    action: &state::Action,
    _action_channel: &Sender<state::ActionWrapper>,
) -> Arc<AgentState> {
    match *action {
        state::Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref _entry) => {
                    // @TODO  add entry to source chain
                    // @see #57
                }
            }
            Arc::new(new_state)
        }
        _ => old_state,
    }
}
