//use error::HolochainError;
use state::*;
use std::{
    sync::{mpsc::*, Arc, RwLock, RwLockReadGuard}, thread, time::Duration,
};


/// Object representing a Holochain app instance.
/// Holds the Event loop and processes it with the redux state model.
//#[derive(Clone)]
pub struct Instance {
    state: Arc<RwLock<State>>,
    action_channel: Sender<ActionWrapper>,
    observer_channel: Sender<Observer>,
}

type ClosureType = Box<FnMut(&State) -> bool + Send>;


/// State Observer that executes a closure everytime the State changes.
pub struct Observer {
    pub sensor: ClosureType,
    pub done: bool,
}

impl Observer {
    fn check(&mut self, state: &State) {
        self.done = (self.sensor)(state);
    }
}

pub static DISPATCH_WITHOUT_CHANNELS: &str = "dispatch called without channels open";

impl Instance {

    /// Stack an Action in the Event Queue
    pub fn dispatch(&mut self, action: Action) -> ActionWrapper {
        return dispatch_action(&self.action_channel, action);
    }

    /// Stack an Action in the Event Queue and block until is has been processed.
    pub fn dispatch_and_wait(&mut self, action: Action) {
        return dispatch_action_and_wait(&self.action_channel, &self.observer_channel, action);
    }

    /// Stack an action in the Event Queue and create an Observer on it with the specified closure
    pub fn dispatch_with_observer<F>(&mut self, action: Action, closure: F)
    where
        F: 'static + FnMut(&State) -> bool + Send,
    {
        return dispatch_action_with_observer(&self.action_channel, &self.observer_channel, action, closure);
    }

    /// Start the Event Loop on a seperate thread
    pub fn start_action_loop(&mut self) {
        let (tx_action, rx_action) = channel::<ActionWrapper>();
        let (tx_observer, rx_observer) = channel::<Observer>();
        self.action_channel = tx_action.clone();
        self.observer_channel = tx_observer.clone();

        let state_mutex = self.state.clone();

        thread::spawn(move || {
            let mut state_observers: Vec<Box<Observer>> = Vec::new();

            loop {
                match rx_action.recv_timeout(Duration::from_millis(400)) {
                    Ok(action_wrapper) => {
                        // Mutate state
                        {
                            let mut state = state_mutex.write().unwrap();
                            *state = state.reduce(action_wrapper, &tx_action, &tx_observer);
                        }

                        // Add new observers
                        while let Ok(observer) = rx_observer.try_recv() {
                            state_observers.push(Box::new(observer));
                        }

                        // Run all observer closures
                        {
                            let state = state_mutex.read().unwrap();
                            state_observers = state_observers
                                .into_iter()
                                .map(|mut observer| {
                                    observer.check(&state);
                                    observer
                                })
                                .filter(|observer| !observer.done)
                                .collect::<Vec<_>>();
                        }
                    }
                    Err(ref _recv_error) => {}
                }
            }
        });
    }

    pub fn new() -> Self {
        let (tx_action, _) = channel();
        let (tx_observer, _) = channel();
        Instance {
            state: Arc::new(RwLock::new(State::new())),
            action_channel: tx_action,
            observer_channel: tx_observer,
        }
    }

    pub fn state(&self) -> RwLockReadGuard<State> {
        self.state.read().unwrap()
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}


/// Send Action to Instance's Event Queue and block until is has been processed.
pub fn dispatch_action_and_wait(action_channel:   &Sender<::state::ActionWrapper>,
                                observer_channel: &Sender<Observer>,
                                action:           Action)
{
    // Wrap Action
    let wrapper = ::state::ActionWrapper::new(action);
    let wrapper_clone = wrapper.clone();

    // Create blocking channel
    let (sender, receiver) = channel::<bool>();

    // Create blocking observer
    let closure = move |state: &State| {
        if state.history.contains(&wrapper_clone) {
            sender
              .send(true)
              .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));
            true
        } else {
            false
        }
    };
    let observer = Observer {
        sensor: Box::new(closure),
        done: false,
    };

    // Send observer to instance
    observer_channel
      .send(observer)
      .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));

    // Send action to instance
    action_channel
      .send(wrapper)
      .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));

    // Block until Observer has sensed the completion of the Action
    receiver
      .recv()
      .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));

}


/// Send Action to the Event Queue and create an Observer for it with the specified closure
pub fn dispatch_action_with_observer<F>(action_channel:   &Sender<::state::ActionWrapper>,
                                        observer_channel: &Sender<Observer>,
                                        action:           Action,
                                        closure:          F)
    where
      F: 'static + FnMut(&State) -> bool + Send,
{
    let observer = Observer {
        sensor: Box::new(closure),
        done: false,
    };

    observer_channel
        .send(observer)
        .expect("observer channel to be open");
    dispatch_action(action_channel, action);
}


/// Send Action to the Event Queue
pub fn dispatch_action(action_channel: &Sender<::state::ActionWrapper>, action: Action) -> ActionWrapper {
    let wrapper = ActionWrapper::new(action);
    action_channel
        .send(wrapper.clone())
        .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));
    wrapper
}