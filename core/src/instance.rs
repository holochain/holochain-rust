use action::ActionWrapper;
use context::Context;
use state::State;
use std::{
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, RwLock, RwLockReadGuard,
    },
    thread,
    time::Duration,
};

pub const RECV_DEFAULT_TIMEOUT_MS: Duration = Duration::from_millis(10000);

/// Object representing a Holochain instance, i.e. a running holochain (DNA + DHT + source-chain)
/// Holds the Event loop and processes it with the redux pattern.
#[derive(Clone)]
pub struct Instance {
    /// The object holding the state. Actions go through the store sequentially.
    state: Arc<RwLock<State>>,
    action_channel: SyncSender<ActionWrapper>,
    observer_channel: SyncSender<Observer>,
}

type ClosureType = Box<FnMut(&State) -> bool + Send>;

/// State Observer that executes a closure everytime the State changes.
pub struct Observer {
    pub sensor: ClosureType,
}

pub static DISPATCH_WITHOUT_CHANNELS: &str = "dispatch called without channels open";

impl Instance {
    pub fn default_channel_buffer_size() -> usize {
        100
    }

    /// get a clone of the action channel
    pub fn action_channel(&self) -> SyncSender<ActionWrapper> {
        self.action_channel.clone()
    }

    /// get a clone of the observer channel
    pub fn observer_channel(&self) -> SyncSender<Observer> {
        self.observer_channel.clone()
    }

    /// Stack an Action in the Event Queue
    ///
    /// # Panics
    ///
    /// Panics if called before `start_action_loop`.
    pub fn dispatch(&mut self, action_wrapper: ActionWrapper) {
        dispatch_action(&self.action_channel, action_wrapper)
    }

    /// Stack an Action in the Event Queue and block until is has been processed.
    ///
    /// # Panics
    ///
    /// Panics if called before `start_action_loop`.
    pub fn dispatch_and_wait(&mut self, action_wrapper: ActionWrapper) {
        dispatch_action_and_wait(&self.action_channel, &self.observer_channel, action_wrapper);
    }

    /// Stack an action in the Event Queue and create an Observer on it with the specified closure
    ///
    /// # Panics
    ///
    /// Panics if called before `start_action_loop`.
    pub fn dispatch_with_observer<F>(&mut self, action_wrapper: ActionWrapper, closure: F)
    where
        F: 'static + FnMut(&State) -> bool + Send,
    {
        dispatch_action_with_observer(
            &self.action_channel,
            &self.observer_channel,
            action_wrapper,
            closure,
        )
    }

    /// Returns recievers for actions and observers that get added to this instance
    fn initialize_channels(&mut self) -> (Receiver<ActionWrapper>, Receiver<Observer>) {
        let (tx_action, rx_action) =
            sync_channel::<ActionWrapper>(Self::default_channel_buffer_size());
        let (tx_observer, rx_observer) =
            sync_channel::<Observer>(Self::default_channel_buffer_size());
        self.action_channel = tx_action.clone();
        self.observer_channel = tx_observer.clone();

        (rx_action, rx_observer)
    }

    pub fn initialize_context(&self, context: Arc<Context>) -> Arc<Context> {
        let mut sub_context = (*context).clone();
        sub_context.set_state(self.state.clone());
        sub_context.action_channel = self.action_channel.clone();
        sub_context.observer_channel = self.observer_channel.clone();
        Arc::new(sub_context)
    }

    /// Start the Event Loop on a seperate thread
    pub fn start_action_loop(&mut self, context: Arc<Context>) {
        let (rx_action, rx_observer) = self.initialize_channels();

        let sync_self = self.clone();
        let sub_context = self.initialize_context(context);

        thread::spawn(move || {
            let mut state_observers: Vec<Observer> = Vec::new();
            for action_wrapper in rx_action {
                state_observers = sync_self.process_action(
                    action_wrapper,
                    state_observers,
                    &rx_observer,
                    &sub_context,
                );
            }
        });
    }

    /// Calls the reducers for an action and calls the observers with the new state
    /// returns the new vector of observers
    pub(crate) fn process_action(
        &self,
        action_wrapper: ActionWrapper,
        mut state_observers: Vec<Observer>,
        rx_observer: &Receiver<Observer>,
        context: &Arc<Context>,
    ) -> Vec<Observer> {
        // Mutate state
        {
            let new_state: State;

            {
                // Only get a read lock first so code in reducers can read state as well
                let state = self
                    .state
                    .read()
                    .expect("owners of the state RwLock shouldn't panic");

                // Create new state by reducing the action on old state
                new_state = state.reduce(context.clone(), action_wrapper);
            }

            // Get write lock
            let mut state = self
                .state
                .write()
                .expect("owners of the state RwLock shouldn't panic");

            // Change the state
            *state = new_state;
        }

        // Add new observers
        state_observers.extend(rx_observer.try_iter());

        // Run all observer closures
        {
            let state = self
                .state
                .read()
                .expect("owners of the state RwLock shouldn't panic");
            let mut i = 0;
            while i != state_observers.len() {
                if (&mut state_observers[i].sensor)(&state) {
                    state_observers.remove(i);
                } else {
                    i += 1;
                }
            }
        }
        state_observers
    }

    /// Creates a new Instance with disconnected channels.
    pub fn new(context: Arc<Context>) -> Self {
        let (tx_action, _) = sync_channel(1);
        let (tx_observer, _) = sync_channel(1);
        Instance {
            state: Arc::new(RwLock::new(State::new(context))),
            action_channel: tx_action,
            observer_channel: tx_observer,
        }
    }

    pub fn from_state(state: State) -> Self {
        let (tx_action, _) = sync_channel(1);
        let (tx_observer, _) = sync_channel(1);
        Instance {
            state: Arc::new(RwLock::new(state)),
            action_channel: tx_action,
            observer_channel: tx_observer,
        }
    }

    pub fn state(&self) -> RwLockReadGuard<State> {
        self.state
            .read()
            .expect("owners of the state RwLock shouldn't panic")
    }
}

/*impl Default for Instance {
    fn default(context:Context) -> Self {
        Self::new(context)
    }
}*/

/// Send Action to Instance's Event Queue and block until is has been processed.
///
/// # Panics
///
/// Panics if the channels passed are disconnected.
pub fn dispatch_action_and_wait(
    action_channel: &SyncSender<ActionWrapper>,
    observer_channel: &SyncSender<Observer>,
    action_wrapper: ActionWrapper,
) {
    // Create blocking channel
    let (sender, receiver) = sync_channel::<()>(1);

    // Create blocking observer
    let observer_action_wrapper = action_wrapper.clone();
    let closure = move |state: &State| {
        if state.history.contains(&observer_action_wrapper) {
            sender
                .send(())
                // the channel stays connected until the first message has been sent
                // if this fails that means that it was called after having returned done=true
                .expect("observer called after done");
            true
        } else {
            false
        }
    };

    dispatch_action_with_observer(&action_channel, &observer_channel, action_wrapper, closure);

    // Block until Observer has sensed the completion of the Action
    receiver.recv().expect(DISPATCH_WITHOUT_CHANNELS);
}

/// Send Action to the Event Queue and create an Observer for it with the specified closure
///
/// # Panics
///
/// Panics if the channels passed are disconnected.
pub fn dispatch_action_with_observer<F>(
    action_channel: &SyncSender<ActionWrapper>,
    observer_channel: &SyncSender<Observer>,
    action_wrapper: ActionWrapper,
    closure: F,
) where
    F: 'static + FnMut(&State) -> bool + Send,
{
    let observer = Observer {
        sensor: Box::new(closure),
    };

    observer_channel
        .send(observer)
        .expect(DISPATCH_WITHOUT_CHANNELS);
    dispatch_action(action_channel, action_wrapper);
}

/// Send Action to the Event Queue
///
/// # Panics
///
/// Panics if the channels passed are disconnected.
pub fn dispatch_action(action_channel: &SyncSender<ActionWrapper>, action_wrapper: ActionWrapper) {
    action_channel
        .send(action_wrapper)
        .expect(DISPATCH_WITHOUT_CHANNELS);
}

#[cfg(test)]
pub mod tests {
    extern crate tempfile;
    extern crate test_utils;
    use self::tempfile::tempdir;
    use super::*;
    use action::{tests::test_action_wrapper_get, Action, ActionWrapper};
    use agent::{
        chain_store::ChainStore,
        state::{ActionResponse, AgentState},
    };
    use context::Context;
    use futures::executor::block_on;
    use holochain_agent::Agent;
    use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use holochain_core_types::{
        cas::content::AddressableContent,
        chain_header::{test_chain_header, ChainHeader},
        entry::ToEntry,
        entry_type::EntryType,
    };
    use holochain_dna::{zome::Zome, Dna};
    use logger::Logger;
    use nucleus::{
        actions::initialize::initialize_application,
        ribosome::{callback::Callback, Defn},
    };
    use persister::SimplePersister;
    use state::State;

    use std::{
        sync::{
            mpsc::{channel, sync_channel},
            Arc, Mutex,
        },
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
        fn dump(&self) -> String {
            format!("{:?}", self.log)
        }
    }

    /// create a test logger
    pub fn test_logger() -> Arc<Mutex<TestLogger>> {
        Arc::new(Mutex::new(TestLogger { log: Vec::new() }))
    }

    /// create a test context and TestLogger pair so we can use the logger in assertions
    pub fn test_context_and_logger(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        let agent = Agent::from(agent_name.to_owned());
        let logger = test_logger();
        (
            Arc::new(
                Context::new(
                    agent,
                    logger.clone(),
                    Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
                    FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
                    EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                        .unwrap(),
                ).unwrap(),
            ),
            logger,
        )
    }

    /// create a test context
    pub fn test_context(agent_name: &str) -> Arc<Context> {
        let (context, _) = test_context_and_logger(agent_name);
        context
    }

    /// create a test context
    pub fn test_context_with_channels(
        agent_name: &str,
        action_channel: &SyncSender<ActionWrapper>,
        observer_channel: &SyncSender<Observer>,
    ) -> Arc<Context> {
        let agent = Agent::from(agent_name.to_owned());
        let logger = test_logger();
        Arc::new(
            Context::new_with_channels(
                agent,
                logger.clone(),
                Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
                action_channel.clone(),
                observer_channel.clone(),
                FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            ).unwrap(),
        )
    }

    pub fn test_context_with_state() -> Arc<Context> {
        let mut context = Context::new(
            Agent::from("Florence".to_string()),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
            EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
        ).unwrap();
        let global_state = Arc::new(RwLock::new(State::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    pub fn test_context_with_agent_state() -> Arc<Context> {
        let file_system =
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap();
        let mut context = Context::new(
            Agent::from("Florence".to_string()),
            test_logger(),
            Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
            file_system.clone(),
            EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
        ).unwrap();
        let chain_store = ChainStore::new(file_system);
        let chain_header = test_chain_header();
        let agent_state = AgentState::new_with_top_chain_header(chain_store, chain_header);
        let state = State::new_with_agent(Arc::new(context.clone()), Arc::new(agent_state));
        let global_state = Arc::new(RwLock::new(state));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    #[test]
    fn default_buffer_size_test() {
        assert_eq!(Context::default_channel_buffer_size(), 100);
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance(dna: Dna) -> Result<Instance, String> {
        test_instance_and_context(dna).map(|tuple| tuple.0)
    }

    /// create a test instance
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_and_context(dna: Dna) -> Result<(Instance, Arc<Context>), String> {
        // Create instance and plug in our DNA
        let context = test_context("jane");
        let mut instance = Instance::new(context.clone());
        instance.start_action_loop(context.clone());
        let context = instance.initialize_context(context);

        block_on(initialize_application(dna.clone(), context.clone()))?;

        assert_eq!(instance.state().nucleus().dna(), Some(dna.clone()));
        assert!(instance.state().nucleus().has_initialized());

        /// fair warning... use test_instance_blank() if you want a minimal instance
        assert!(
            !dna.zomes.clone().is_empty(),
            "Empty zomes = No genesis = infinite loops below!"
        );

        // @TODO abstract and DRY this out
        // @see https://github.com/holochain/holochain-rust/issues/195
        while instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::InitApplication(_) => true,
                _ => false,
            })
            .is_none()
        {
            println!("Waiting for InitApplication");
            sleep(Duration::from_millis(10))
        }

        while instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert!(
                        entry.entry_type() == &EntryType::AgentId
                            || entry.entry_type() == &EntryType::Dna
                    );
                    true
                }
                _ => false,
            })
            .is_none()
        {
            println!("Waiting for Commit for genesis");
            sleep(Duration::from_millis(10))
        }

        while instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::ReturnInitializationResult(_) => true,
                _ => false,
            })
            .is_none()
        {
            println!("Waiting for ReturnInitializationResult");
            sleep(Duration::from_millis(10))
        }
        Ok((instance, context))
    }

    /// create a test instance with a blank DNA
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_blank() -> Instance {
        let mut dna = Dna::new();
        dna.zomes.insert("".to_string(), Zome::default());
        dna.uuid = "2297b5bc-ef75-4702-8e15-66e0545f3482".into();
        test_instance(dna).expect("Blank instance could not be initialized!")
    }

    #[test]
    /// This tests calling `process_action`
    /// with an action that dispatches no new ones.
    /// It tests that the desired effects do happen
    /// to the state and that no observers or actions
    /// are sent on the passed channels.
    pub fn can_process_action() {
        let mut instance = Instance::new(test_context("jason"));

        let context = test_context("jane");
        let (rx_action, rx_observer) = instance.initialize_channels();

        let action_wrapper = test_action_wrapper_get();
        let new_observers = instance.process_action(
            action_wrapper.clone(),
            Vec::new(), // start with no observers
            &rx_observer,
            &context,
        );

        // test that the get action added no observers or actions
        assert!(new_observers.is_empty());

        let rx_action_is_empty = match rx_action.try_recv() {
            Err(::std::sync::mpsc::TryRecvError::Empty) => true,
            _ => false,
        };
        assert!(rx_action_is_empty);

        let rx_observer_is_empty = match rx_observer.try_recv() {
            Err(::std::sync::mpsc::TryRecvError::Empty) => true,
            _ => false,
        };
        assert!(rx_observer_is_empty);

        // Borrow the state lock
        let state = instance.state();
        // Clone the agent Arc
        let actions = state.agent().actions();
        let response = actions
            .get(&action_wrapper)
            .expect("action and reponse should be added after Get action dispatch");

        assert_eq!(response, &ActionResponse::GetEntry(None));
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
        let mut instance = Instance::new(test_context("jason"));
        instance.start_action_loop(test_context("jane"));

        let dna = Dna::new();
        let (sender, receiver) = sync_channel(1);
        instance.dispatch_with_observer(
            ActionWrapper::new(Action::InitApplication(dna.clone())),
            move |state: &State| match state.nucleus().dna() {
                Some(dna) => {
                    sender
                        .send(dna)
                        // the channel stays connected until the first message has been sent
                        // if this fails that means that it was called after having returned done=true
                        .expect("observer called after done");
                    true
                }
                None => false,
            },
        );

        let stored_dna = receiver.recv().expect("observer dropped before done");

        assert_eq!(dna, stored_dna);
    }

    #[test]
    /// tests that we can dispatch an action and block until it completes
    fn can_dispatch_and_wait() {
        let mut instance = Instance::new(test_context("jason"));
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(
            instance.state().nucleus().status(),
            ::nucleus::state::NucleusStatus::New
        );

        let dna = Dna::new();

        let action = ActionWrapper::new(Action::InitApplication(dna.clone()));
        instance.start_action_loop(test_context("jane"));

        // the initial state is not intialized
        assert_eq!(
            instance.state().nucleus().status(),
            ::nucleus::state::NucleusStatus::New
        );

        instance.dispatch_and_wait(action);
        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(
            instance.state().nucleus().status(),
            ::nucleus::state::NucleusStatus::Initializing
        );
    }

    #[test]
    /// tests that an unimplemented genesis allows the nucleus to initialize
    /// @TODO is this right? should return unimplemented?
    /// @see https://github.com/holochain/holochain-rust/issues/97
    fn test_missing_genesis() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome",
            Callback::Genesis.capability().as_str(),
            None,
        );

        let instance = test_instance(dna);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a valid genesis allows the nucleus to initialize
    fn test_genesis_ok() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome",
            Callback::Genesis.capability().as_str(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis") (param $p0 i32) (result i32)
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
        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a failed genesis prevents the nucleus from initializing
    fn test_genesis_err() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome",
            Callback::Genesis.capability().as_str(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis") (param $p0 i32) (result i32)
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
        assert!(instance.is_err());
        assert_eq!(instance.err().unwrap(), "1337");
    }

    /// Committing a DnaEntry to source chain should work
    #[test]
    fn can_commit_dna() {
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex");
        let dna = test_utils::create_test_dna_with_wat("test_zome", "test_cap", None);
        let dna_entry = dna.to_entry();
        let commit_action = ActionWrapper::new(Action::Commit(dna_entry.clone()));

        // Set up instance and process the action
        let instance = Instance::new(test_context("jason"));
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_action, state_observers, &rx_observer, &context);

        // Check if AgentIdEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(entry.entry_type(), &EntryType::Dna);
                    assert_eq!(entry.content(), dna_entry.content());
                    true
                }
                _ => false,
            });
    }

    /// Committing an AgentIdEntry to source chain should work
    #[test]
    fn can_commit_agent() {
        // Create Context, Agent and Commit AgentIdEntry Action
        let context = test_context("alex");
        let agent_entry = context.agent.to_entry();
        let commit_agent_action = ActionWrapper::new(Action::Commit(agent_entry.clone()));

        // Set up instance and process the action
        let instance = Instance::new(test_context("jason"));
        let state_observers: Vec<Observer> = Vec::new();
        let (_, rx_observer) = channel::<Observer>();
        instance.process_action(commit_agent_action, state_observers, &rx_observer, &context);

        // Check if AgentIdEntry is found
        assert_eq!(1, instance.state().history.iter().count());
        instance
            .state()
            .history
            .iter()
            .find(|aw| match aw.action() {
                Action::Commit(entry) => {
                    assert_eq!(entry.entry_type(), &EntryType::AgentId,);
                    assert_eq!(entry.content(), agent_entry.content());
                    true
                }
                _ => false,
            });
    }
}
