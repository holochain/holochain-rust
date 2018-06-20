use error::HolochainError;
use state::*;
use std::collections::VecDeque;
use std::sync::mpsc::*;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Instance {
    state: State,
    pending_actions: VecDeque<Action>,
    action_channel: Sender<Action>,
    observer_channel: Sender<Observer>,
}

type ClosureType = Box<FnMut(&State) -> bool + Send>;

pub struct Observer {
    sensor: ClosureType,
    done: bool,
}

impl Observer {
    fn check(&mut self, state: &State) {
        self.done = (self.sensor)(state);
    }
}

impl Instance {
    pub fn dispatch(&mut self, action: Action) {
        self.action_channel
            .send(action)
            .expect("action channel to be open");
    }

    pub fn dispatch_with_observer<F>(&mut self, action: Action, closure: F)
    where
        F: 'static + FnMut(&State) -> bool + Send,
    {
        let observer = Observer {
            sensor: Box::new(closure),
            done: false,
        };

        self.observer_channel
            .send(observer)
            .expect("observer channel to be open");
        self.dispatch(action);
    }

    pub fn pending_actions(&self) -> &VecDeque<Action> {
        &self.pending_actions
    }

    pub fn consume_next_action(&mut self) -> Result<(), HolochainError> {
        if !self.pending_actions.is_empty() {
            let result = self.pending_actions.pop_front();
            match result {
                None => {
                    return Err(HolochainError::ErrorGeneric(
                        "nothing to consume".to_string(),
                    ))
                }
                Some(action) => self.state = self.state.clone().reduce(&action, &self.action_channel),
            }
        }
        Ok(())
    }

    pub fn start_action_loop(&mut self) {
        let (tx_action, rx_action) = channel();
        //let (tx_state, rx_state) = channel();
        let (tx_observer, rx_observer) = channel::<Observer>();
        self.action_channel = tx_action.clone();
        self.observer_channel = tx_observer.clone();

        thread::spawn(move || {
            let mut state = State::new();
            let mut state_observers: Vec<Box<Observer>> = Vec::new();

            loop {
                match rx_action.recv_timeout(Duration::from_millis(400)) {
                    Ok(action) => {
                        state = state.clone().reduce(&action, &tx_action);
                        //tx_state.send(state.clone());
                    }
                    Err(ref _recv_error) => {}
                }

                match rx_observer.try_recv() {
                    Ok(observer) => {
                        state_observers.push(Box::new(observer));
                    }
                    Err(ref _recv_error) => {}
                }

                state_observers = state_observers
                    .into_iter()
                    .map(|mut observer| {
                        observer.check(&state);
                        observer
                    })
                    .filter(|observer| !observer.done)
                    .collect::<Vec<_>>();
            }
        });
    }

    pub fn new() -> Self {
        let (tx_action, _) = channel();
        let (tx_observer, _) = channel();
        Instance {
            state: State::new(),
            pending_actions: VecDeque::new(),
            action_channel: tx_action,
            observer_channel: tx_observer,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}
