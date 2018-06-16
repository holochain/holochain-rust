pub mod keys;

use self::keys::Keys;
use source_chain::memory::SourceChain;
use common::entry::Entry;
use state;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    keys: Option<Keys>,
    source_chain: Option<Box<SourceChain>>,
}

impl AgentState {
    pub fn new() -> Self {
        AgentState {
            keys: None,
            source_chain: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Commit(Entry),
}

pub fn reduce(old_state: Rc<AgentState>, action: &state::Action) -> Rc<AgentState> {
    match *action {
        state::Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref _entry) => {}
            }
            Rc::new(new_state)
        }
        _ => old_state,
    }
}
