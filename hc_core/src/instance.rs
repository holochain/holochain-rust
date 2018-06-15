use state::*;
use std::collections::VecDeque;
use std::sync::mpsc::*;
use std::thread;
use std::time::Duration;
use std::sync::Arc;

pub struct Instance {
    state: State,
    pending_actions: VecDeque<Action>,
    action_channel: Sender<Action>
}

impl Instance {
    pub fn dispatch(&mut self, action: Action) {
        self.pending_actions.push_back(action);
    }

    pub fn pending_actions(&self) -> &VecDeque<Action> {
        &self.pending_actions
    }

    pub fn consume_next_action(&mut self) {
        if self.pending_actions.len() > 0 {
            let action = self.pending_actions.pop_front().unwrap();
            self.state = self.state.clone().reduce(&action);
        }
    }

    pub fn start_action_loop(&mut self) {
        let (tx_action, rx_action) = channel();
        self.action_channel = tx_action.clone();

        thread::spawn(move || {
            let mut state = State::create();
            //self.state = &state;
            match rx_action.recv_timeout(Duration::from_millis(400)) {
                Ok(action) => state = state.clone().reduce(&action),
                Err(ref recv_error) => {}
            }
        });
    }

    pub fn create() -> Self {
        let (sender, receiver) = channel();
        Instance {
            state: State::create(),
            pending_actions: VecDeque::new(),
            action_channel: sender
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}
