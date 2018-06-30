extern crate snowflake;

use agent::AgentState;
use nucleus::NucleusState;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
#[allow(unknown_lints)]
#[allow(large_enum_variant)]
pub enum Action<'a> {
    Agent(::agent::Action<'a>),
    Network(::network::Action),
    Nucleus(::nucleus::Action),
}

#[derive(Clone, Debug)]
pub struct ActionWrapper<'a> {
    pub action: Action<'a>,
    pub id: snowflake::ProcessUniqueId,
}

impl<'a> ActionWrapper<'a> {
    pub fn new(a: Action) -> Self {
        ActionWrapper {
            action: a,
            id: snowflake::ProcessUniqueId::new(),
        }
    }
}

impl<'a> PartialEq for ActionWrapper<'a> {
    fn eq(&self, other: &ActionWrapper) -> bool {
        self.id == other.id
    }
}

impl<'a> Eq for ActionWrapper<'a> {}

impl<'a> Hash for ActionWrapper<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct State<'a> {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState<'a>>,
    pub history: HashSet<ActionWrapper<'a>>,
}

impl<'a> State<'a> {
    pub fn new() -> Self {
        State {
            nucleus: Arc::new(NucleusState::new()),
            agent: Arc::new(AgentState::new()),
            history: HashSet::new(),
        }
    }

    pub fn reduce(
        &self,
        action_wrapper: ActionWrapper,
        action_channel: &Sender<ActionWrapper>,
    ) -> Self {
        let mut new_state = State {
            nucleus: ::nucleus::reduce(
                Arc::clone(&self.nucleus),
                &action_wrapper.action,
                action_channel,
            ),
            agent: ::agent::reduce(
                Arc::clone(&self.agent),
                &action_wrapper.action,
                action_channel,
            ),
            history: self.history.clone(),
        };

        new_state.history.insert(action_wrapper);
        new_state
    }

    pub fn nucleus(&self) -> Arc<NucleusState> {
        Arc::clone(&self.nucleus)
    }

    pub fn agent(&self) -> Arc<AgentState> {
        Arc::clone(&self.agent)
    }
}

/*
TODO: write macro for DRY reducer functions
macro_rules! reducer {
    ($func_name:ident) => (
        fn reducer(old_state: Rc<$state_type>, action: &_Action) -> Rc<$state_type>  {
            // The `stringify!` macro converts an `ident` into a string.
            println!("You called {:?}()",
                     stringify!($func_name));
        }
    )
}
*/
