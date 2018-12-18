use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::memory::EavMemoryStorage,
};
use holochain_container_api::{
    config::{
        load_configuration, AgentConfiguration, Configuration, DnaConfiguration,
        InstanceConfiguration, LoggerConfiguration, StorageConfiguration,
    },
    container::Container,
    Holochain,
};
use holochain_core::{
    action::Action,
    context::{mock_network_config, Context as HolochainContext},
    logger::Logger,
    persister::SimplePersister,
    signal::{signal_channel, Signal, SignalReceiver},
};
use holochain_core_types::{agent::AgentId, dna::Dna, json::JsonString};
use neon::{context::Context, prelude::*};
use snowflake::ProcessUniqueId;
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    path::PathBuf,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex, RwLock,
    },
    thread,
};
use tempfile::tempdir;

use crate::config::*;

pub type JsCallback = JsFunction;

/// A set of closures, each of which checks for a certain condition to be met
/// (usually for a certain action to be seen). When the condition specified by the closure
/// is met, that closure is removed from the set of checks.
///
/// When the set of checks goes from non-empty to empty, send a message via `tx`
/// to the BlockTask on the other side
struct CallFxChecker {
    tx: SyncSender<WaiterMsg>,
    conditions: HashSet<CallFxCondition>, // maybe RefCell
}

// Could maybe reduce this to HashSet<ActionWrapper> if we only need to check for
// simple existence of one of any possible ActionWrappers
type CallFxCondition = Box<Fn(&ActionWrapper) -> bool>;

impl CallFxChecker {
    pub fn new(callback: JsCallback) -> Self {
        let (tx, rx) = sync_channel(1);
        let task = CallBlockingTask { rx };
        task.schedule(callback);
        Self {
            tx,
            conditions: HashSet::new(),
        }
    }

    pub fn add<F>(&mut self, f: F) -> ()
    where
        F: Fn(&ActionWrapper) -> bool,
    {
        self.conditions.insert(Box::new(f));
    }

    pub fn run_checks(&mut self, aw: &ActionWrapper) {
        let was_empty = self.conditions.is_empty();
        for condition in self.conditions {
            if (condition(aw)) {
                self.conditions.remove(condition);
            }
        }
        if self.conditions.is_empty() && !was_empty {
            self.tx.send(WaiterMsg::Stop);
        }
    }
}

/// A simple Task that blocks until it receives a message.
/// This is used to trigger a JS Promise resolution when a ZomeFnCall's
/// side effects have all completed.
struct CallBlockingTask {
    pub rx: Receiver<WaiterMsg>,
}

impl Task for CallBlockingTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        while let Ok(sig) = self.rx.recv() {
            match sig {
                WaiterMsg::Stop => break,
            }
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsUndefined> {
        result.map(|_| cx.undefined())
    }
}

/// A singleton which runs in a Task and is the receiver for the Signal channel.
/// - handles incoming `ZomeFnCall`s, attaching and activating a new `CallFxChecker`
/// - handles incoming Signals, running all `CallFxChecker` closures
pub struct Waiter {
    checkers: HashMap<ZomeFnCall, CallFxChecker>,
    current: Option<ZomeFnCall>, // maybe RefCell
    callback_rx: CallbackReceiver,
}

type CallbackReceiver = Receiver<JsCallback>;

impl Waiter {
    pub fn new(callback_rx: CallbackReceiver) -> Self {
        Self {
            checkers: HashMap::new(),
            current: None,
            callback_rx,
        }
    }

    pub fn process_signal(sig: Signal) {
        match sig {
            Signal::Internal(aw) => {
                match aw.action() {
                    Action::ExecuteZomeFunction(call) => {
                        let callback = self.callback_rx.recv();
                        self.add_call(call, callback);
                    }
                    Action::Commit((entry, _)) => match entry {
                        App(_, _) => self
                            .current_checker()
                            .unwrap()
                            .add(|aw| aw.action == Action::Hold(entry.clone())),
                    },
                    Action::SendDirectMessage(data) => {
                        let DirectMessageData { msg_id } = data;
                        let possible_actions = &[
                            ResolveDirectConnection(msg_id),
                            SendDirectMessageTimeout(msg_id),
                        ];
                        self.current_checker()
                            .unwrap()
                            .add(|aw| possible_actions.contains(aw))
                    }
                    _ => (),
                }

                self.run_checks(aw);
            }
            _ => (),
        }
    }

    fn run_checks(&mut self, aw: &ActionWrapper) {
        for (_call, checker) in self.checkers {
            checker.run_checks(aw);
        }
    }

    fn current_checker(&mut self) -> Option<&mut CallFxChecker> {
        self.current.and_then(|call| self.checkers.get_mut(call))
    }

    fn add_call(&mut self, call: ZomeFnCall, callback: JsCallback) {
        self.checkers.insert(call, CallFxChecker::new(callback));
        self.current = Some(call);
    }
}

enum WaiterMsg {
    Stop,
}

pub struct HabitatSignalTask {
    signal_rx: SignalReceiver,
    waiter: RefCell<Waiter>,
}

impl HabitatSignalTask {
    pub fn new(signal_rx: SignalReceiver, callback_rx: CallbackReceiver) -> Self {
        let this = Self {
            signal_rx,
            waiter: Waiter::new(callback_rx),
        };
        this
    }
}

impl Task for HabitatSignalTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<(), String> {
        use std::io::{self, Write};
        while let Ok(sig) = self.signal_rx.recv() {
            self.waiter.borrow_mut().process_signal(sig);
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
        Ok(cx.number(17))
    }
}
