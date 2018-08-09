//use error::HolochainError;
use context::Context;
use state::*;
use std::{
    sync::{mpsc::*, Arc, RwLock, RwLockReadGuard},
    thread,
    time::Duration,
};

pub const REDUX_LOOP_TIMEOUT_MS: u64 = 400;
pub const REDUX_DEFAULT_TIMEOUT_MS: u64 = 2000;

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
    /// get a clone of the action channel
    pub fn action_channel(&self) -> Sender<ActionWrapper> {
        self.action_channel.clone()
    }

    /// get a clone of the observer channel
    pub fn observer_channel(&self) -> Sender<Observer> {
        self.observer_channel.clone()
    }

    /// Stack an Action in the Event Queue
    pub fn dispatch(&mut self, action: Action) -> ActionWrapper {
        dispatch_action(&self.action_channel, action)
    }

    /// Stack an Action in the Event Queue and block until is has been processed.
    pub fn dispatch_and_wait(&mut self, action: Action) {
        dispatch_action_and_wait(&self.action_channel, &self.observer_channel, action);
    }

    /// Stack an action in the Event Queue and create an Observer on it with the specified closure
    pub fn dispatch_with_observer<F>(&mut self, action: Action, closure: F)
    where
        F: 'static + FnMut(&State) -> bool + Send,
    {
        dispatch_action_with_observer(
            &self.action_channel,
            &self.observer_channel,
            action,
            closure,
        )
    }

    /// Start the Event Loop on a seperate thread
    pub fn start_action_loop(&mut self, context: Arc<Context>) {
        let (tx_action, rx_action) = channel::<ActionWrapper>();
        let (tx_observer, rx_observer) = channel::<Observer>();
        self.action_channel = tx_action.clone();
        self.observer_channel = tx_observer.clone();

        let state_mutex = self.state.clone();

        thread::spawn(move || {
            let mut state_observers: Vec<Box<Observer>> = Vec::new();

            // @TODO this should all be callable outside the loop so that deterministic tests that
            // don't rely on time can be written
            // @see https://github.com/holochain/holochain-rust/issues/169
            loop {
                match rx_action.recv_timeout(Duration::from_millis(REDUX_LOOP_TIMEOUT_MS)) {
                    Ok(action_wrapper) => {
                        // Mutate state
                        {
                            let mut state = state_mutex.write().unwrap();
                            *state = state.reduce(
                                context.clone(),
                                action_wrapper,
                                &tx_action,
                                &tx_observer,
                            );
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
pub fn dispatch_action_and_wait(
    action_channel: &Sender<::state::ActionWrapper>,
    observer_channel: &Sender<Observer>,
    action: Action,
) {
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
pub fn dispatch_action_with_observer<F>(
    action_channel: &Sender<::state::ActionWrapper>,
    observer_channel: &Sender<Observer>,
    action: Action,
    closure: F,
) where
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
pub fn dispatch_action(
    action_channel: &Sender<::state::ActionWrapper>,
    action: Action,
) -> ActionWrapper {
    let wrapper = ActionWrapper::new(action);
    action_channel
        .send(wrapper.clone())
        .unwrap_or_else(|_| panic!(DISPATCH_WITHOUT_CHANNELS));
    wrapper
}

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    use super::Instance;
    use context::Context;
    use holochain_agent::Agent;
    use holochain_dna::{zome::capabilities::ReservedCapabilityNames, Dna};
    use logger::Logger;
    use nucleus::Action::InitApplication;
    use persister::SimplePersister;
    use state::{Action::Nucleus, State};
    use std::{
        sync::{mpsc::channel, Arc, Mutex},
        thread::sleep,
        time::Duration,
    };

    #[derive(Clone, Debug)]
    pub struct TestLogger {
        pub log: Vec<String>,
    }

    impl Logger for TestLogger {
        fn log(&mut self, msg: String) {
            self.log.push(msg);
        }
    }

    /// create a test logger
    pub fn test_logger() -> Arc<Mutex<TestLogger>> {
        Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
    }

    /// create a test context and TestLogger pair so we can use the logger in assertions
    pub fn test_context_and_logger(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        let agent = Agent::from_string(agent_name);
        let logger = test_logger();
        (
            Arc::new(Context {
                agent,
                logger: logger.clone(),
                persister: Arc::new(Mutex::new(SimplePersister::new())),
            }),
            logger,
        )
    }

    /// create a test context
    pub fn test_context(agent_name: &str) -> Arc<Context> {
        let (context, _) = test_context_and_logger(agent_name);
        context
    }

    /// create a test instance
    pub fn test_instance(dna: Dna) -> Instance {
        // Create instance and plug in our DNA
        let mut instance = Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop(test_context("jane"));
        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Wait for Init to finish
        while instance.state().history.len() < 4 {
            // TODO - #21
            // This println! should be converted to either a call to the app logger, or to the core debug log.
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10))
        }

        instance
    }

    /// create a test instance with a blank DNA
    pub fn test_instance_blank() -> Instance {
        test_instance(Dna::new())
    }

    #[test]
    /// This test shows how to call dispatch with a closure that should run
    /// when the action results in a state change.  Note that the observer closure
    /// needs to return a boolean to indicate that it has successfully observed what
    /// it intends to observe.  It will keep getting called as the state changes until
    /// it returns true.
    /// Note also that for this test we create a channel to send something (in this case
    /// the dna) back over, just so that the test will block until the closure is successfully
    /// run and the assert will actually run.  If we put the assert inside the closure
    /// the test thread could complete before the closure was called.
    fn can_dispatch_with_observer() {
        let mut instance = Instance::new();
        instance.start_action_loop(test_context("jane"));

        let dna = Dna::new();
        let (sender, receiver) = channel();
        instance.dispatch_with_observer(
            Nucleus(InitApplication(dna.clone())),
            move |state: &State| match state.nucleus().dna() {
                Some(dna) => {
                    sender.send(dna).expect("test channel must be open");
                    return true;
                }
                None => return false,
            },
        );

        let stored_dna = receiver.recv().unwrap();

        assert_eq!(dna, stored_dna);
    }

    #[test]
    /// tests that we can dispatch an action and block until it completes
    fn can_dispatch_and_wait() {
        let mut instance = Instance::new();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(
            instance.state().nucleus().status(),
            ::nucleus::NucleusStatus::New
        );

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop(test_context("jane"));

        // the initial state is not intialized
        assert!(instance.state().nucleus().has_initialized() == false);

        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Wait for Init to finish
        while instance.state().history.len() < 2 {
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10));
        }
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that an unimplemented genesis allows the nucleus to initialize
    /// @TODO is this right? should return unimplemented?
    /// @see https://github.com/holochain/holochain-rust/issues/97
    fn test_missing_genesis() {
        let mut dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            "test_cap".to_string(),
            None,
        );
        dna.zomes[0].capabilities[0].name = ReservedCapabilityNames::LifeCycle.as_str().to_string();

        let instance = test_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a valid genesis allows the nucleus to initialize
    fn test_genesis_ok() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis_dispatch") (param $p0 i32) (result i32)
                    i32.const 0
                )
                (data (i32.const 0)
                    ""
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let instance = test_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a failed genesis prevents the nucleus from initializing
    fn test_genesis_err() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis_dispatch") (param $p0 i32) (result i32)
                    i32.const 4
                )
                (data (i32.const 0)
                    "1337"
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let instance = test_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized() == false);
    }
}
