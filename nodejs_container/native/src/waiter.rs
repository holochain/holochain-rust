use colored::*;
use holochain_core::{
    action::{Action, ActionWrapper},
    network::direct_message::DirectMessage,
    nucleus::ZomeFnCall,
    signal::{Signal, SignalReceiver},
};
use neon::{context::Context, prelude::*};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        mpsc::{Receiver, RecvTimeoutError, SyncSender},
        Arc, Mutex,
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
        println!(
            "\n*** Condition {}: {} -> {}",
            "ADDED".green(),
            self.conditions.len() - 1,
            self.conditions.len()
        );
    }

    pub fn run_checks(&mut self, aw: &ActionWrapper) -> bool {
        let was_empty = self.conditions.is_empty();
        let size = self.conditions.len();
        self.conditions.retain(|condition| !condition(aw));
        if size != self.conditions.len() {
            println!(
                "\n*** Condition {}: {} -> {}",
                "REMOVED".red(),
                size,
                size - 1
            );
        }
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
    println!("{}:\n{}\n", "(((LOG)))".bold(), msg);
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
        let do_log = match sig {
            Signal::Internal(ref aw) => {
                let aw = aw.clone();
                let do_log = match aw.action().clone() {
                    Action::ExecuteZomeFunction(call) => {
                        match self.sender_rx.try_recv() {
                            Ok(sender) => {
                                self.add_call(call.clone(), sender);
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
                        // TODO: is there a possiblity that this can get messed up if the same
                        // entry is committed multiple times?
                        Some(checker) => {
                            checker.add(move |aw| *aw.action() == Action::Hold(entry.clone()));
                            true
                        }
                        None => false,
                    },
                    Action::SendDirectMessage(data) => {
                        let msg_id = data.msg_id;
                        match (self.current_checker(), data.message) {
                            (Some(checker), DirectMessage::Custom(_)) => {
                                checker.add(move |aw| {
                                    [
                                        Action::ResolveDirectConnection(msg_id.clone()),
                                        Action::SendDirectMessageTimeout(msg_id.clone()),
                                    ]
                                    .contains(aw.action())
                                });
                                true
                            }
                            _ => false,
                        }
                    }
                    Action::ResolveDirectConnection(_) => true,
                    Action::SendDirectMessageTimeout(_) => true,
                    Action::Hold(_) => true,
                    _ => false,
                };

                self.run_checks(&aw);
                do_log
            }
            _ => false,
        };
        if do_log {
            println!("{} {:?}", ":::SIG".cyan(), sig);
        }
    }

    fn run_checks(&mut self, aw: &ActionWrapper) {
        let size = self.checkers.len();
        self.checkers.retain(|_, checker| checker.run_checks(aw));
        if size != self.checkers.len() {
            println!(
                "\n{}: {} -> {}",
                "Num checkers".italic(),
                size,
                self.checkers.len()
            );
        }
    }

    fn current_checker(&mut self) -> Option<&mut CallFxChecker> {
        self.current
            .clone()
            .and_then(move |call| self.checkers.get_mut(&call))
    }

    fn add_call(&mut self, call: ZomeFnCall, tx: ControlSender) {
        let checker = CallFxChecker::new(tx);

        log("Waiter: add_call...");
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
    pub fn new(
        signal_rx: SignalReceiver,
        sender_rx: Receiver<ControlSender>,
        is_running: Arc<Mutex<bool>>,
    ) -> Self {
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
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        while *self.is_running.lock().unwrap() {
            // TODO: could use channels more intelligently to stop immediately
            // rather than waiting for timeout, but it's more complicated.
            match self.signal_rx.recv_timeout(Duration::from_millis(250)) {
                Ok(sig) => self.waiter.borrow_mut().process_signal(sig),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(err) => return Err(err.to_string()),
            }
        }

        println!("{}", "\nHOW ABOUT LET'S STOP?".red().bold());

        for (_, checker) in self.waiter.borrow_mut().checkers.iter_mut() {
            println!("{}", "Shutting down lingering checker...".magenta().bold());
            checker.shutdown();
        }
        println!("{}", "ONLY NOW ARE WE ACTUALLY sTOPPED\n".magenta().bold());
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsUndefined> {
        println!("{}", "Background task shutting down...".bold().magenta());
        result.or_else(|e| {
            let error_string = cx.string(format!("unable to shut down background task: {}", e));
            cx.throw(error_string)
        })?;
        println!("{}", "...with no errors".bold().magenta());
        Ok(cx.undefined())
    }
}
