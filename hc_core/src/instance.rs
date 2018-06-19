use error::HolochainError;
use state::*;
use std::collections::VecDeque;

#[derive(Clone)]
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

    pub fn consume_next_action(&mut self) -> Result<(), HolochainError> {
        if self.pending_actions.len() > 0 {
            let result = self.pending_actions.pop_front();
            match result {
                None => {
                    return Err(HolochainError::ErrorGeneric(
                        "nothing to consume".to_string(),
                    ))
                }
                Some(action) => self.state = self.state.clone().reduce(&action),
            }
        }
        Ok(())
    }

    pub fn new() -> Self {
        Instance {
            state: State::new(),
            pending_actions: VecDeque::new(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}
