use holochain_core::{
    action::{Action, ActionWrapper},
    nucleus::ZomeFnCall,
    signal::{Signal, SignalReceiver},
};
use holochain_core_types::{
    entry::Entry,
    link::{link_add::LinkAdd, Link},
};
use neon::{context::Context, prelude::*};
use std::{
    cell::RefCell,
    collections::{HashMap},
    sync::{
        mpsc::{Receiver, SyncSender, RecvTimeoutError},
        Arc, Mutex, RwLock,
    },
    time::Duration,
};

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

    pub fn run_checks(&mut self, aw: &ActionWrapper) -> bool {
        let was_empty = self.conditions.is_empty();
        let size = self.conditions.len();
        self.conditions.retain(|condition| !condition(aw));
        println!("{}/{}", size, self.conditions.len());
        if self.conditions.is_empty() && !was_empty {
            self.stop();
            return false;
        } else {
            return true;
        }
    }

    pub fn shutdown(&mut self) {
        self.conditions.clear();
        self.stop();
    }

    fn stop(&mut self) {
        self.tx.send(ControlMsg::Stop).unwrap();
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

fn log(msg: &str) {
    println!("\nLOG:\n{}\n", msg);
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
        let do_log = match sig.clone() {
            Signal::Internal(aw) => {
                let do_log = match aw.action().clone() {
                    Action::ExecuteZomeFunction(call) => {
                        match self.sender_rx.try_recv() {
                            Ok(sender) => {
                                self.add_call(call.clone(), sender);
                                log("Waiter: add_call");
                                self.current_checker().unwrap().add(move |aw| {
                                    if let Action::ReturnZomeFunctionResult(ref r) = *aw.action() {
                                        r.call() == call
                                    } else {
                                        false
                                    }
                                });
                            }
                            Err(_) => {
                                self.deactivate_current();
                                log("Waiter: deactivate_current");
                            }
                        }
                        true
                    }
                    Action::ReturnZomeFunctionResult(_) => true,
                    Action::Commit((entry, _)) => match self.current_checker() {
                        Some(checker) => {
                            checker.add(move |aw| *aw.action() == Action::Hold(entry.clone()));
                            true
                        }
                        None => false,
                    },
                    Action::AddLink(link) => match self.current_checker() {
                        Some(checker) => {
                            let entry = Entry::LinkAdd(LinkAdd::new(
                                link.base(),
                                link.target(),
                                link.tag(),
                            ));
                            checker.add(move |aw| *aw.action() == Action::Hold(entry.clone()));
                            true
                        }
                        None => false,
                    },
                    // Action::SendDirectMessage(data) => {
                    //     let msg_id = data.msg_id;
                    //     match self.current_checker() {
                    //         Some(checker) => checker.add(move |aw| {
                    //             [
                    //                 Action::ResolveDirectConnection(msg_id.clone()),
                    //                 Action::SendDirectMessageTimeout(msg_id.clone()),
                    //             ]
                    //             .contains(aw.action())
                    //         }),
                    //         None => (),
                    //     }
                    // },
                    Action::Hold(_) => true,
                    _ => false,
                };

                self.run_checks(&aw);
                do_log
            }
            _ => false,
        };
        if do_log {
            let mut s = format!(":::SIG {:?}", sig);
            s.truncate(300);
            println!("{}", s);
        }
    }

    fn run_checks(&mut self, aw: &ActionWrapper) {
        self.checkers.retain(|_, checker| checker.run_checks(aw));
    }

    fn current_checker(&mut self) -> Option<&mut CallFxChecker> {
        self.current
            .clone()
            .and_then(move |call| self.checkers.get_mut(&call))
    }

    fn add_call(&mut self, call: ZomeFnCall, tx: ControlSender) {
        let checker = CallFxChecker::new(tx);
        self.checkers.insert(call.clone(), checker);
        self.current = Some(call);
    }

    fn deactivate_current(&mut self) {
        self.current = None;
    }
}

pub enum ControlMsg {
    Stop,
}

pub struct MainBackgroundTask {
    signal_rx: SignalReceiver,
    waiter: RefCell<Waiter>,
    is_running: Arc<Mutex<bool>>,
}

impl MainBackgroundTask {
    pub fn new(signal_rx: SignalReceiver, sender_rx: Receiver<ControlSender>, is_running: Arc<Mutex<bool>>) -> Self {
        let this = Self {
            signal_rx,
            waiter: RefCell::new(Waiter::new(sender_rx)),
            is_running,
        };
        this
    }
}

impl Task for MainBackgroundTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<(), String> {
        while *self.is_running.lock().unwrap() {
            // TODO: could use channels more intelligently to stop immediately 
            // rather than waiting for timeout, but it's complicated.
            match self.signal_rx.recv_timeout(Duration::from_millis(250)) {
                Ok(sig) => self.waiter.borrow_mut().process_signal(sig),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(err) => return Err(err.to_string()),
            }
        }
        for (_, checker) in self.waiter.borrow_mut().checkers.iter_mut() {
            checker.shutdown();
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, _result: Result<(), String>) -> JsResult<JsNumber> {
        Ok(cx.number(17))
    }
}
