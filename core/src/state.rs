use agent::AgentState;
use instance::Observer;
use nucleus::NucleusState;
use snowflake;
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::{mpsc::Sender, Arc},
};

#[derive(Clone, Debug, PartialEq)]
#[allow(unknown_lints)]
#[allow(large_enum_variant)]
pub enum Action {
    Agent(::agent::Action),
    Network(::network::Action),
    Nucleus(::nucleus::Action),
}

#[derive(Clone, Debug)]
pub struct ActionWrapper {
    pub action: Action,
    pub id: snowflake::ProcessUniqueId,
}

impl ActionWrapper {
    pub fn new(a: Action) -> Self {
        ActionWrapper {
            action: a,
            id: snowflake::ProcessUniqueId::new(),
        }
    }
}

impl PartialEq for ActionWrapper {
    fn eq(&self, other: &ActionWrapper) -> bool {
        self.id == other.id
    }
}

impl Eq for ActionWrapper {}

impl Hash for ActionWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct State {
    nucleus: Arc<NucleusState>,
    agent: Arc<AgentState>,
    pub history: HashSet<ActionWrapper>,
}

impl State {
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
        observer_channel: &Sender<Observer>,
    ) -> Self {
        let mut new_state = State {
            nucleus: ::nucleus::reduce(
                Arc::clone(&self.nucleus),
                &action_wrapper.action,
                action_channel,
                observer_channel,
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
