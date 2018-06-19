use state::*;
use std::collections::VecDeque;

pub struct Instance {
    state: State,
    pending_actions: VecDeque<Action>,
}

impl Instance {
    pub fn dispatch(&mut self, action: Action) {
        self.pending_actions.push_back(action);
    }

    pub fn pending_actions(&self) -> &VecDeque<Action> {
        &self.pending_actions
    }

    pub fn consume_next_action(&mut self) {
        if !self.pending_actions.is_empty() {
            let action = self.pending_actions.pop_front().unwrap();
            self.state = self.state.clone().reduce(&action);
        }
    }

    pub fn create() -> Self {
        Instance {
            state: State::create(),
            pending_actions: VecDeque::new(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}
