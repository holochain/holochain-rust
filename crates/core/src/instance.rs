use crate::{
    action::{Action, ActionWrapper},
    consistency::ConsistencyModel,
    context::{ActionReceiver, ActionSender, Context},
    dht::actions::{
        queue_holding_workflow::queue_holding_workflow,
        remove_queued_holding_workflow::remove_queued_holding_workflow,
    },
    network,
    persister::Persister,
    scheduled_jobs,
    signal::Signal,
    state::{State, StateWrapper},
    workflows::{application, run_holding_workflow},
};
#[cfg(test)]
use crate::{
    network::actions::initialize_network::initialize_network_with_spoofed_dna,
    nucleus::actions::initialize::initialize_chain,
};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use crossbeam_channel::{unbounded, Receiver, Sender};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
};
use holochain_locksmith::RwLock;
#[cfg(test)]
use holochain_persistence_api::cas::content::Address;
use holochain_tracing::{self as ht, channel::lax_send_wrapped};
use snowflake::ProcessUniqueId;
use std::{
    sync::{
        atomic::Ordering::{self, Relaxed},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub const RECV_DEFAULT_TIMEOUT_MS: Duration = Duration::from_millis(10000);
pub const RETRY_VALIDATION_DURATION_MIN: Duration = Duration::from_millis(500);
pub const RETRY_VALIDATION_DURATION_MAX: Duration = Duration::from_secs(60 * 60);

/// Object representing a Holochain instance, i.e. a running holochain (DNA + DHT + source-chain)
/// Holds the Event loop and processes it with the redux pattern.
#[derive(Clone)]
pub struct Instance {
    /// The object holding the state. Actions go through the store sequentially.
    state: Arc<RwLock<StateWrapper>>,
    action_channel: Option<ActionSender>,
    observer_channel: Option<Sender<Observer>>,
    scheduler_handle: Option<Arc<ScheduleHandle>>,
    persister: Option<Arc<RwLock<dyn Persister>>>,
    consistency_model: ConsistencyModel,
    kill_switch: Option<Sender<()>>,
    kill_switch_holding: Option<Sender<()>>,
}

/// State Observer that executes a closure everytime the State changes.
pub struct Observer {
    pub ticker: Sender<()>,
}

pub static DISPATCH_WITHOUT_CHANNELS: &str = "dispatch called without channels open";

#[autotrace]
impl Instance {
    /// This is initializing and starting the redux action loop and adding channels to dispatch
    /// actions and observers to the context
    pub(in crate::instance) fn inner_setup(&mut self, context: Arc<Context>) -> Arc<Context> {
        let (rx_action, rx_observer) = self.initialize_channels();
        let context = self.initialize_context(context);
        let mut scheduler = Scheduler::new();
        scheduler
            .every(10.seconds())
            .run(scheduled_jobs::create_state_dump_callback(context.clone()));
        scheduler
            .every(1.second())
            .run(scheduled_jobs::create_timeout_callback(context.clone()));
        scheduler
            .every(30.seconds())
            .run(scheduled_jobs::create_state_pruning_callback(
                context.clone(),
            ));
        self.scheduler_handle = Some(Arc::new(scheduler.watch_thread(Duration::from_millis(10))));

        self.persister = Some(context.persister.clone());

        self.start_action_loop(context.clone(), rx_action, rx_observer);
        self.start_holding_loop(context.clone());

        context
    }

    /// This is calling inner_setup and running the initialization workflow which makes sure that
    /// the chain gets initialized if dna is Some.
    /// If dna is None it is assumed the chain is already initialized, i.e. we are loading a chain.
    pub fn initialize(
        &mut self,
        dna: Option<Dna>,
        context: Arc<Context>,
    ) -> HcResult<Arc<Context>> {
        let context = self.inner_setup(context);
        context.block_on(application::initialize(self, dna, context.clone()))
    }

    /// This function is only needed in tests to create integration tests in which an instance
    /// tries to publish invalid entries.
    /// The DNA needs to be spoofed then so that we can emulate a hacked node that does not
    /// run the right validation checks locally but actually commits and publishes invalid data.
    #[cfg(test)]
    pub fn initialize_with_spoofed_dna(
        &mut self,
        dna: Dna,
        spoofed_dna_address: Address,
        context: Arc<Context>,
    ) -> HcResult<Arc<Context>> {
        let context = self.inner_setup(context);
        context.block_on(async {
            initialize_chain(dna.clone(), &context).await?;
            initialize_network_with_spoofed_dna(spoofed_dna_address, &context).await
        })?;
        Ok(context)
    }

    /// Only needed in tests to check that the initialization (and other workflows) fail
    /// with the right error message if no DNA is present.
    #[cfg(test)]
    pub fn initialize_without_dna(&mut self, context: Arc<Context>) -> Arc<Context> {
        self.inner_setup(context)
    }

    // @NB: these three getters smell bad because previously Instance and Context had SyncSenders
    // rather than Option<SyncSenders>, but these would be initialized by default to broken channels
    // which would panic if `send` was called upon them. These `expect`s just bring more visibility to
    // that potential failure mode.
    // @see https://github.com/holochain/holochain-rust/issues/739
    fn action_channel(&self) -> &ActionSender {
        self.action_channel
            .as_ref()
            .expect("Action channel not initialized")
    }

    pub fn observer_channel(&self) -> &Sender<Observer> {
        self.observer_channel
            .as_ref()
            .expect("Observer channel not initialized")
    }

    /// Stack an Action in the Event Queue
    ///
    /// # Panics
    ///
    /// Panics if called before `start_action_loop`.
    pub fn dispatch(&mut self, action_wrapper: ActionWrapper) {
        dispatch_action(self.action_channel(), action_wrapper)
    }

    /// Returns recievers for actions and observers that get added to this instance
    fn initialize_channels(&mut self) -> (ActionReceiver, Receiver<Observer>) {
        let (tx_action, rx_action) = unbounded::<ht::SpanWrap<ActionWrapper>>();
        let (tx_observer, rx_observer) = unbounded::<Observer>();
        self.action_channel = Some(tx_action.into());
        self.observer_channel = Some(tx_observer);

        (rx_action, rx_observer)
    }

    pub fn initialize_context(&self, context: Arc<Context>) -> Arc<Context> {
        let mut sub_context = (*context).clone();
        sub_context.set_state(self.state.clone());
        sub_context.action_channel = self.action_channel.clone();
        sub_context.observer_channel = self.observer_channel.clone();
        Arc::new(sub_context)
    }

    /// Start the Event Loop on a separate thread
    pub fn start_action_loop(
        &mut self,
        context: Arc<Context>,
        rx_action: ActionReceiver,
        rx_observer: Receiver<Observer>,
    ) {
        self.stop_action_loop();

        let mut sync_self = self.clone();
        let sub_context = self.initialize_context(context);

        let (kill_sender, kill_receiver) = crossbeam_channel::unbounded();
        self.kill_switch = Some(kill_sender);
        let instance_is_alive = sub_context.instance_is_alive.clone();
        instance_is_alive.store(true, Ordering::Relaxed);

        let _ = thread::Builder::new()
            .name(format!(
                "action_loop/{}",
                ProcessUniqueId::new().to_string()
            ))
            .spawn(move || {
                let mut state_observers: Vec<Observer> = Vec::new();
                let mut unprocessed_action: Option<ht::SpanWrap<ActionWrapper>> = None;
                while kill_receiver.try_recv().is_err() {
                    // TODO: I changed a clone() to a take() because I couldn't see how this would work otherwise. Was it right??
                    if let Some(action_wrapper) = unprocessed_action.take().or_else(|| rx_action.recv_timeout(Duration::from_secs(1)).ok()) {
                        // Add new observers
                        state_observers.extend(rx_observer.try_iter());
                        let action = action_wrapper.action();
                        // Ping can happen often, and should be as lightweight as possible
                        let should_process = *action != Action::Ping;
                        if should_process {
                            match sync_self.process_action(&action_wrapper, &sub_context) {
                                Ok(()) => {
                                    let tag = ht::Tag::new("action", format!("{:?}", action));
                                    let _guard = action_wrapper.follower_(&sub_context.tracer, "action_loop thread", |s| s.tag(tag).start()).map(|span| {

                                        ht::push_span(span)
                                    });
                                    sync_self.emit_signals(&sub_context, &action_wrapper);
                                    // Tick all observers and remove those that have lost their receiving part
                                    state_observers= state_observers
                                        .into_iter()
                                        .filter(|observer| observer.ticker.send(()).is_ok())
                                        .collect();
                                },
                                Err(HolochainError::Timeout) => {
                                    warn!("Instance::process_action() couldn't get lock on state. Trying again next loop.");
                                    unprocessed_action = Some(action_wrapper);
                                },
                                Err(e) => {
                                    error!("Instance::process_action() returned unexpected error: {:?}", e);
                                    unprocessed_action = Some(action_wrapper);
                                }
                            };

                        }
                    }
                }
                instance_is_alive.store(false, Relaxed);
            });
    }

    pub fn stop_action_loop(&self) {
        if let Some(ref kill_switch) = self.kill_switch {
            let _ = kill_switch.send(());
        }
        if let Some(ref kill_switch) = self.kill_switch_holding {
            let _ = kill_switch.send(());
        }
    }

    /// Calls the reducers for an action and calls the observers with the new state
    /// returns the new vector of observers
    pub(crate) fn process_action(
        &self,
        action_wrapper: &ActionWrapper,
        context: &Arc<Context>,
    ) -> Result<(), HolochainError> {
        let _trace_guard = ht::push_span(context.tracer.span("ROOT: process_action").start().into());
        context.redux_wants_write.store(true, Relaxed);
        // Mutate state
        {
            let new_state: StateWrapper;

            // Get write lock
            let mut state = self
                .state
                .try_write_until(Instant::now().checked_add(Duration::from_secs(10)).unwrap())
                .ok_or_else(|| HolochainError::Timeout)?;

            new_state = state.reduce(action_wrapper.clone());

            // Change the state
            *state = new_state;

            if let Err(e) = self.save(&state) {
                log_error!(
                    context,
                    "instance/process_action: could not save state: {:?}",
                    e
                );
            } else {
                log_trace!(
                    context,
                    "reduce/process_actions: reducing {:?}",
                    action_wrapper
                );
            }
        }

        context.redux_wants_write.store(false, Relaxed);

        Ok(())
    }

    fn start_holding_loop(&mut self, context: Arc<Context>) {
        let (kill_sender, kill_receiver) = crossbeam_channel::unbounded();
        self.kill_switch_holding = Some(kill_sender);
        thread::Builder::new()
            .name(format!(
                "holding_loop/{}",
                ProcessUniqueId::new().to_string()
            ))
            .spawn(move || {
                while kill_receiver.try_recv().is_err() {
                    log_trace!(context, "Checking holding queue...");
                    loop {
                        // TODO: TRACING: it would be ideal to be able to associate a tracing Span with each queued holding workflow.
                        // To do this, we'd need to store a Span in each item of the DhtStore::queued_holding_workflows.
                        // However, Span is not Clone, and the entire DhtStore needs to be Cloned.
                        // So, the span has to be cut short here until we stop cloning the state.

                        let dht_store = context
                            .state()
                            .expect("Couldn't get state in run_pending_validations")
                            .dht();
                        let maybe_holding_workflow = dht_store.next_queued_holding_workflow();
                        if let Some((pending, maybe_delay)) = maybe_holding_workflow {
                            log_debug!(context, "Found queued validation: {:?}", pending);
                            // NB: If for whatever reason we pop_next_holding_workflow anywhere else other than here,
                            // we can run into a race condition.
                            context.block_on(remove_queued_holding_workflow(
                                pending.clone(),
                                context.clone(),
                            ));

                            let c = context.clone();
                            let pending = pending.clone();

                            let closure = async move || {
                                match run_holding_workflow(pending.clone(), c.clone()).await {
                                    // If we couldn't run the validation due to unresolved dependencies,
                                    // we have to re-add this entry at the end of the queue:
                                    Err(HolochainError::ValidationPending) => {
                                        // And with a delay so we are not trying to re-validate many times per second.
                                        let mut delay = maybe_delay
                                            .map(|old_delay| {
                                                // Exponential back-off:
                                                // If this was delayed before we double the delay.
                                                old_delay * 2
                                            })
                                            .unwrap_or(RETRY_VALIDATION_DURATION_MIN);

                                        // Cap delay with max duration
                                        if delay > RETRY_VALIDATION_DURATION_MAX {
                                            delay = RETRY_VALIDATION_DURATION_MAX
                                        }

                                        queue_holding_workflow(
                                            Arc::new(pending.same()),
                                            Some(delay),
                                            c.clone(),
                                        )
                                        .await
                                    }
                                    Err(e) => log_error!(
                                        c,
                                        "Error running holding workflow for {:?}: {:?}",
                                        pending,
                                        e,
                                    ),
                                    Ok(()) => log_info!(c, "Successfully processed: {:?}", pending),
                                }
                            };
                            let future = closure();
                            context.spawn_task(future);
                        } else {
                            break;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            })
            .expect("Could not spawn holding thread");
    }

    pub(crate) fn emit_signals(&mut self, context: &Context, action_wrapper: &ActionWrapper) {
        if let Some(tx) = context.signal_tx() {
            // @TODO: if needed for performance, could add a filter predicate here
            // to prevent emitting too many unneeded signals
            let trace_signal = Signal::Trace(action_wrapper.clone());
            tx.send(trace_signal).unwrap_or_else(|e| {
                log_warn!(
                    context,
                    "reduce: Signal channel is closed! No signals can be sent ({:?}).",
                    e
                );
            });

            if let Some(signal) = self
                .consistency_model
                .process_action(action_wrapper.action())
            {
                tx.send(Signal::Consistency(signal.into()))
                    .unwrap_or_else(|e| {
                        log_warn!(
                            context,
                            "reduce: Signal channel is closed! No signals can be sent ({:?}).",
                            e
                        );
                    });
            }
        }
    }

    /// Creates a new Instance with no channels set up.
    pub fn new(context: Arc<Context>) -> Self {
        Instance {
            state: Arc::new(RwLock::new(StateWrapper::new(context.clone()))),
            action_channel: None,
            observer_channel: None,
            scheduler_handle: None,
            persister: None,
            consistency_model: ConsistencyModel::new(context),
            kill_switch: None,
            kill_switch_holding: None,
        }
    }

    pub fn from_state(state: State, context: Arc<Context>) -> Self {
        Instance {
            state: Arc::new(RwLock::new(StateWrapper::from(state))),
            action_channel: None,
            observer_channel: None,
            scheduler_handle: None,
            persister: None,
            consistency_model: ConsistencyModel::new(context),
            kill_switch: None,
            kill_switch_holding: None,
        }
    }

    pub fn state(&self) -> StateWrapper {
        self.state
            .read()
            .expect("owners of the state RwLock shouldn't panic")
            .clone()
    }

    pub fn save(&self, state: &StateWrapper) -> HcResult<()> {
        self.persister
            .as_ref()
            .ok_or_else(|| HolochainError::new("Instance::save() called without persister set."))?
            .try_write()
            .ok_or_else(|| HolochainError::new("Could not get lock on persister"))?
            .save(&state)
    }

    #[allow(clippy::needless_lifetimes)]
    #[no_autotrace]
    pub async fn shutdown_network(&self) -> HcResult<()> {
        network::actions::shutdown::shutdown(
            self.state.clone(),
            self.action_channel.as_ref().unwrap(),
        )
        .await
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // TODO: this is already performed in Holochain::stop explicitly,
        // can we get rid of one or the other?
        let _ = self.shutdown_network();
        self.stop_action_loop();
        self.state.write().unwrap().drop_inner_state();
    }
}

/*impl Default for Instance {
    fn default(context:Context) -> Self {
        Self::new(context)
    }
}*/

/// Send Action to the Event Queue
///
/// # Panics
///
/// Panics if the channels passed are disconnected.
pub fn dispatch_action(action_channel: &ActionSender, action_wrapper: ActionWrapper) {
    lax_send_wrapped(action_channel.clone(), action_wrapper, "dispatch_action");
}

#[cfg(test)]
pub mod tests {
    use self::tempfile::tempdir;
    use super::*;
    use crate::{
        action::{tests::test_action_wrapper_commit, Action, ActionWrapper},
        agent::{
            chain_store::ChainStore,
            state::{AgentActionResponse, AgentState},
        },
        context::{test_memory_network_config, Context},
        logger::{test_logger, TestLogger},
    };
    use holochain_core_types::{
        agent::AgentId,
        chain_header::test_chain_header,
        dna::{zome::Zome, Dna},
        entry::{entry_type::EntryType, test_entry},
    };
    use holochain_locksmith::{Mutex, RwLock};
    use holochain_persistence_api::cas::content::AddressableContent;
    use holochain_persistence_file::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
    use tempfile;
    use test_utils;

    use crate::persister::SimplePersister;

    use std::{sync::Arc, thread::sleep, time::Duration};

    use test_utils::mock_signing::registered_test_agent;

    use crate::nucleus::state::NucleusStatus;
    use holochain_core_types::entry::Entry;
    use holochain_json_api::json::JsonString;
    use holochain_persistence_lmdb::{cas::lmdb::LmdbStorage, eav::lmdb::EavLmdbStorage};
    use holochain_persistence_mem::{cas::memory::MemoryStorage, eav::memory::EavMemoryStorage};

    /// create a test context and TestLogger pair so we can use the logger in assertions
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_and_logger(
        agent_name: &str,
        network_name: Option<&str>,
    ) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        test_context_and_logger_with_in_memory_network(agent_name, network_name)
    }

    /// create a test context and TestLogger pair so we can use the logger in assertions
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_and_logger_with_in_memory_network(
        agent_name: &str,
        network_name: Option<&str>,
    ) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        let agent = registered_test_agent(agent_name);
        let content_storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let meta_storage = Arc::new(RwLock::new(EavMemoryStorage::new()));
        let logger = test_logger();
        (
            Arc::new(Context::new(
                "Test-context-and-logger-instance",
                agent,
                Arc::new(RwLock::new(SimplePersister::new(content_storage.clone()))),
                content_storage.clone(),
                content_storage.clone(),
                meta_storage,
                test_memory_network_config(network_name),
                None,
                None,
                false,
                holochain_metrics::config::MetricPublisherConfig::default()
                .create_metric_publisher(),
                Arc::new(ht::null_tracer()),
            )),
            logger,
        )
    }

    /// create a test context
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context(agent_name: &str, network_name: Option<&str>) -> Arc<Context> {
        let (context, _) = test_context_and_logger(agent_name, network_name);
        context
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_with_memory_network(
        agent_name: &str,
        network_name: Option<&str>,
    ) -> Arc<Context> {
        let (context, _) = test_context_and_logger_with_in_memory_network(agent_name, network_name);
        context
    }

    /// create a test context
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_with_channels(
        agent_name: &str,
        action_channel: &ActionSender,
        observer_channel: &Sender<Observer>,
        network_name: Option<&str>,
    ) -> Arc<Context> {
        let agent = AgentId::generate_fake(agent_name);
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        Arc::new(
            Context::new_with_channels(
                "Test-context-with-channels-instance",
                agent,
                Arc::new(RwLock::new(SimplePersister::new(file_storage.clone()))),
                Some(action_channel.clone()),
                None,
                Some(observer_channel.clone()),
                file_storage.clone(),
                Arc::new(RwLock::new(
                    EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                        .unwrap(),
                )),
                // TODO should bootstrap nodes be set here?
                test_memory_network_config(network_name),
                false,
                Arc::new(RwLock::new(
                    holochain_metrics::DefaultMetricPublisher::default(),
                )),
                Arc::new(ht::null_tracer()),
            )
            .unwrap(),
        )
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_with_state(network_name: Option<&str>) -> Arc<Context> {
        let file_storage = Arc::new(RwLock::new(
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
        ));
        let mut context = Context::new(
            "test-context-with-state-instance",
            registered_test_agent("Florence"),
            Arc::new(RwLock::new(SimplePersister::new(file_storage.clone()))),
            file_storage.clone(),
            file_storage.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            // TODO BLOCKER should bootstrap nodes be set here?
            test_memory_network_config(network_name),
            None,
            None,
            false,
            Arc::new(RwLock::new(
                holochain_metrics::DefaultMetricPublisher::default(),
            )),
            Arc::new(ht::null_tracer()),
        );
        let global_state = Arc::new(RwLock::new(StateWrapper::new(Arc::new(context.clone()))));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_context_with_agent_state(network_name: Option<&str>) -> Arc<Context> {
        let file_system =
            FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap();
        let cas = Arc::new(RwLock::new(file_system.clone()));
        let mut context = Context::new(
            "test-context-with-agent-state-instance",
            registered_test_agent("Florence"),
            Arc::new(RwLock::new(SimplePersister::new(cas.clone()))),
            cas.clone(),
            cas.clone(),
            Arc::new(RwLock::new(
                EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                    .unwrap(),
            )),
            // TODO BLOCKER should bootstrap nodes be set here?
            test_memory_network_config(network_name),
            None,
            None,
            false,
            Arc::new(RwLock::new(
                holochain_metrics::DefaultMetricPublisher::default(),
            )),
            Arc::new(ht::null_tracer()),
        );
        let chain_store = ChainStore::new(cas.clone());
        let chain_header = test_chain_header();
        let agent_state = AgentState::new_with_top_chain_header(
            chain_store,
            Some(chain_header),
            context.agent_id.address(),
        );
        let state = StateWrapper::new_with_agent(Arc::new(context.clone()), agent_state);
        let global_state = Arc::new(RwLock::new(state));
        context.set_state(global_state.clone());
        Arc::new(context)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance(dna: Dna, network_name: Option<&str>) -> Result<Instance, String> {
        test_instance_and_context(dna, network_name).map(|tuple| tuple.0)
    }

    /// create a canonical test instance
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_and_context(
        dna: Dna,
        network_name: Option<&str>,
    ) -> Result<(Instance, Arc<Context>), String> {
        test_instance_and_context_by_name(dna, "jane", network_name)
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_and_context_by_name(
        dna: Dna,
        name: &str,
        network_name: Option<&str>,
    ) -> Result<(Instance, Arc<Context>), String> {
        test_instance_and_context_with_memory_network_nodes(dna, name, network_name)
    }

    /// create a test instance
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_and_context_with_memory_network_nodes(
        dna: Dna,
        name: &str,
        network_name: Option<&str>,
    ) -> Result<(Instance, Arc<Context>), String> {
        // Create instance and plug in our DNA
        let context = test_context_with_memory_network(name, network_name);
        let mut instance = Instance::new(context.clone());
        let context = instance.initialize(Some(dna.clone()), context.clone())?;

        assert_eq!(instance.state().nucleus().dna(), Some(dna.clone()));
        assert!(instance.state().nucleus().has_initialized());

        // fair warning... use test_instance_blank() if you want a minimal instance
        assert!(
            !dna.zomes.clone().is_empty(),
            "Empty zomes = No init = infinite loops below!"
        );

        loop {
            if let NucleusStatus::Initialized(_) = instance.state().nucleus().status {
                break;
            } else {
                println!("Waiting for initialized status");
                sleep(Duration::from_millis(10))
            }
        }

        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_type() == EntryType::Dna));

        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_type() == EntryType::AgentId));

        Ok((instance, context))
    }

    /// create a test instance with a blank DNA
    #[cfg_attr(tarpaulin, skip)]
    pub fn test_instance_blank() -> Instance {
        let mut dna = Dna::new();
        dna.zomes.insert("".to_string(), Zome::empty());
        dna.uuid = "2297b5bc-ef75-4702-8e15-66e0545f3482".into();
        test_instance(dna, None).expect("Blank instance could not be initialized!")
    }

    #[test]
    /// This tests calling `process_action`
    /// with an action that dispatches no new ones.
    /// It tests that the desired effects do happen
    /// to the state and that no observers or actions
    /// are sent on the passed channels.
    pub fn can_process_action() {
        let netname = Some("can_process_action");
        let mut instance = Instance::new(test_context("jason", netname));
        let context = instance.initialize_context(test_context("jane", netname));
        let (rx_action, rx_observer) = instance.initialize_channels();

        let action_wrapper = test_action_wrapper_commit();
        instance
            .process_action(&action_wrapper, &context)
            .expect("process_action should run without error");

        let rx_action_is_empty = match rx_action.try_recv() {
            Err(crossbeam_channel::TryRecvError::Empty) => true,
            _ => false,
        };
        assert!(rx_action_is_empty);

        let rx_observer_is_empty = match rx_observer.try_recv() {
            Err(crossbeam_channel::TryRecvError::Empty) => true,
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

        assert_eq!(
            response.response(),
            &AgentActionResponse::Commit(Ok(test_entry().address()))
        );
    }

    #[test]
    /// tests that an unimplemented init allows the nucleus to initialize
    /// @TODO is this right? should return unimplemented?
    /// @see https://github.com/holochain/holochain-rust/issues/97
    fn test_missing_init() {
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);

        let instance = test_instance(dna, None);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a valid init allows the nucleus to initialize
    fn test_init_ok() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome",
            Some(
                r#"
            (module
                (memory (;0;) 1)
                (func (export "init") (param $p0 i64) (result i64)
                    i64.const 0
                )
                (data (i32.const 0)
                    ""
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let maybe_instance = test_instance(dna, Some("test_init_ok"));
        assert!(maybe_instance.is_ok());

        let instance = maybe_instance.unwrap();
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    /// tests that a failed init prevents the nucleus from initializing
    fn test_init_err() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome",
            Some(
                r#"
            (module
                (memory (;0;) 1)
                (func (export "init") (param $p0 i64) (result i64)
                    i64.const 9
                )
                (data (i32.const 0)
                    "1337.0"
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let instance = test_instance(dna, None);
        assert!(instance.is_err());
        assert_eq!(
            instance.err().unwrap(),
            String::from(
                "At least one zome init returned error: [(\"test_zome\", \"\\\"Init\\\"\")]"
            )
        );
    }

    /// Committing a DnaEntry to source chain should work
    #[test]
    fn can_commit_dna() {
        let netname = Some("can_commit_dna");
        // Create Context, Agent, Dna, and Commit AgentIdEntry Action
        let context = test_context("alex", netname);
        let dna = test_utils::create_test_dna_with_wat("test_zome", None);
        let dna_entry = Entry::Dna(Box::new(dna));
        let commit_action = ActionWrapper::new(Action::Commit((dna_entry.clone(), None, vec![])));

        // Set up instance and process the action
        let instance = Instance::new(test_context("jason", netname));
        let context = instance.initialize_context(context);
        instance
            .process_action(&commit_action, &context)
            .expect("process_action should run without error");

        // Check if AgentIdEntry is found
        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_address() == dna_entry.address()))
    }

    /// Committing an AgentIdEntry to source chain should work
    #[test]
    fn can_commit_agent() {
        let netname = Some("can_commit_agent");
        // Create Context, Agent and Commit AgentIdEntry Action
        let context = test_context("alex", netname);
        let agent_entry = Entry::AgentId(context.agent_id.clone());
        let commit_agent_action =
            ActionWrapper::new(Action::Commit((agent_entry.clone(), None, vec![])));

        // Set up instance and process the action
        let instance = Instance::new(context.clone());
        let context = instance.initialize_context(context);
        instance
            .process_action(&commit_agent_action, &context)
            .expect("process_action should run without error");

        assert!(instance
            .state()
            .agent()
            .iter_chain()
            .any(|header| *header.entry_address() == agent_entry.address()))
    }

    /// create a test context using LMDB storage
    fn test_context_lmdb(
        agent_name: &str,
        network_name: Option<&str>,
        cas_initial_mmap: Option<usize>,
    ) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        let agent = registered_test_agent(agent_name);

        let cas_dir = tempdir().expect("Could not create a tempdir for CAS testing");
        let eav_dir = tempdir().expect("Could not create a tempdir for CAS testing");

        let content_storage = Arc::new(RwLock::new(LmdbStorage::new(
            cas_dir.path(),
            cas_initial_mmap,
        )));
        let meta_storage = Arc::new(RwLock::new(EavLmdbStorage::new(eav_dir.path(), None)));
        let logger = test_logger();
        (
            Arc::new(Context::new(
                "Test-context-lmdb",
                agent,
                Arc::new(RwLock::new(SimplePersister::new(content_storage.clone()))),
                content_storage.clone(),
                content_storage.clone(),
                meta_storage,
                test_memory_network_config(network_name),
                None,
                None,
                false,
                Arc::new(RwLock::new(
                    holochain_metrics::DefaultMetricPublisher::default(),
                )),
                Arc::new(ht::null_tracer()),
            )),
            logger,
        )
    }

    #[test]
    fn lmdb_large_entry_test() {
        let megabytes = 1024 * 1024;
        let initial_mmap_size = 1 * megabytes;
        let (context, _) =
            test_context_lmdb("alice", Some("lmdb_stress_test"), Some(initial_mmap_size));

        // Set up instance
        let instance = Instance::new(context.clone());
        let context = instance.initialize_context(context);

        fn test_entry(i: u32, reps: usize) -> Entry {
            let data: String = std::iter::repeat(format!("{}", i)).take(reps).collect();
            Entry::App("test-entry".into(), JsonString::from_json(&data))
        }

        // write an entry larger than the initial mmap
        let entry = test_entry(0, 3 * initial_mmap_size);
        let commit_agent_action = ActionWrapper::new(Action::Commit((entry.clone(), None, vec![])));

        instance
            .process_action(&commit_agent_action, &context)
            .unwrap();

        // ensure it was added
        let dht = context.dht_storage.read().unwrap();
        assert!(dht.contains(&entry.address()).unwrap());
    }
}
