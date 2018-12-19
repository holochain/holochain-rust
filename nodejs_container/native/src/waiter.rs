use holochain_core::{
    action::{Action, ActionWrapper},
    context::{mock_network_config, Context as HolochainContext},
    nucleus::ZomeFnCall,
    signal::{signal_channel, Signal, SignalReceiver},
};
use holochain_core_types::entry::Entry;
use neon::{context::Context, prelude::*};
use snowflake::ProcessUniqueId;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex, RwLock,
    },
};

use crate::config::*;

pub type JsCallback = Handle<'static, JsFunction>;

type ControlSender = SyncSender<ControlMsg>;
type ControlReceiver = Receiver<ControlMsg>;

/// A set of closures, each of which checks for a certain condition to be met
/// (usually for a certain action to be seen). When the condition specified by the closure
/// is met, that closure is removed from the set of checks.
///
/// When the set of checks goes from non-empty to empty, send a message via `tx`
/// to the BlockTask on the other side
struct CallFxChecker {
    tx: ControlSender,
    conditions: Vec<CallFxCondition>, // maybe RefCell
}

// Could maybe reduce this to HashSet<ActionWrapper> if we only need to check for
// simple existence of one of any possible ActionWrappers
type CallFxCondition = Box<Fn(&ActionWrapper) -> bool + 'static + Send>;

impl CallFxChecker {
    pub fn new(tx: ControlSender) -> Self {
        Self {
            tx,
            conditions: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, f: F) -> ()
    where
        F: Fn(&ActionWrapper) -> bool + 'static + Send,
    {
        self.conditions.push(Box::new(f));
    }

    pub fn run_checks(&mut self, aw: &ActionWrapper) {
        let was_empty = self.conditions.is_empty();
        self.conditions.retain(|condition| !condition(aw));
        if self.conditions.is_empty() && !was_empty {
            self.tx.send(ControlMsg::Stop).unwrap();
        }
    }
}

/// A simple Task that blocks until it receives a message.
/// This is used to trigger a JS Promise resolution when a ZomeFnCall's
/// side effects have all completed.
pub struct CallBlockingTask {
    pub rx: ControlReceiver,
}

impl Task for CallBlockingTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        while let Ok(sig) = self.rx.recv() {
            match sig {
                ControlMsg::Stop => break,
            }
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsUndefined> {
        result.map(|_| cx.undefined()).or_else(|e| {
            let error_string = cx.string(format!("unable to initialize habitat: {}", e));
            cx.throw(error_string)
        })
    }
}

/// A singleton which runs in a Task and is the receiver for the Signal channel.
/// - handles incoming `ZomeFnCall`s, attaching and activating a new `CallFxChecker`
/// - handles incoming Signals, running all `CallFxChecker` closures
pub struct Waiter {
    checkers: HashMap<ZomeFnCall, CallFxChecker>,
    current: Option<ZomeFnCall>, // maybe RefCell
    sender_rx: Receiver<ControlSender>,
}

impl Waiter {
    pub fn new(sender_rx: Receiver<ControlSender>) -> Self {
        Self {
            checkers: HashMap::new(),
            current: None,
            sender_rx,
        }
    }

    pub fn process_signal(&mut self, sig: Signal) {
        match sig {
            Signal::Internal(aw) => {
                match aw.action().clone() {
                    Action::ExecuteZomeFunction(call) => {
                        let sender = self.sender_rx.recv().unwrap();
                        self.add_call(call.clone(), sender);
                    }
                    Action::Commit((entry, _)) => match entry {
                        Entry::App(_, _) => self
                            .current_checker()
                            .unwrap()
                            .add(move |aw| *aw.action() == Action::Hold(entry.clone())),
                        _ => (),
                    },
                    Action::SendDirectMessage(data) => {
                        let msg_id = data.msg_id;
                        self.current_checker().unwrap().add(move |aw| {
                            [
                                Action::ResolveDirectConnection(msg_id.clone()),
                                Action::SendDirectMessageTimeout(msg_id.clone()),
                            ]
                            .contains(aw.action())
                        })
                    }
                    _ => (),
                }

                self.run_checks(&aw);
            }
            _ => (),
        }
    }

    fn run_checks(&mut self, aw: &ActionWrapper) {
        for (_, mut checker) in self.checkers.iter_mut() {
            checker.run_checks(aw);
        }
    }

    fn current_checker(&mut self) -> Option<&mut CallFxChecker> {
        self.current
            .clone()
            .and_then(move |call| self.checkers.get_mut(&call))
    }

    fn add_call(&mut self, call: ZomeFnCall, tx: ControlSender) {
        self.checkers.insert(call.clone(), CallFxChecker::new(tx));
        self.current = Some(call);
    }
}

pub enum ControlMsg {
    Stop,
}

pub struct HabitatSignalTask {
    signal_rx: SignalReceiver,
    waiter: RefCell<Waiter>,
}

impl HabitatSignalTask {
    pub fn new(signal_rx: SignalReceiver, sender_rx: Receiver<ControlSender>) -> Self {
        let this = Self {
            signal_rx,
            waiter: RefCell::new(Waiter::new(sender_rx)),
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
