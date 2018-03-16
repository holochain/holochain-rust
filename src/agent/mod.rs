pub mod keys;
pub mod source_chain;

use self::keys::Keys;
use self::source_chain::SourceChain;
use common::entry::Entry;
use state::Action as _Action;
use std::rc::Rc;
use std::cmp::PartialEq;

#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    keys : Option<Keys>,
    source_chain : Option<Box<SourceChain>>
}

impl AgentState {
    pub fn create() -> Self {
        AgentState {
            keys: None,
            source_chain: None
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Commit(Entry),

}

pub fn reduce(old_state: Rc<AgentState>, action: &_Action) -> Rc<AgentState> {
    match *action {
        _Action::Agent(ref agent_action) => {
            let mut new_state: AgentState = (*old_state).clone();
            match *agent_action {
                Action::Commit(ref entry) => {

                }
            }
            Rc::new(new_state)
        },
        _ => old_state
    }
}
