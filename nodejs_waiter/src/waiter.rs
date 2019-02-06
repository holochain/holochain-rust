use colored::*;
use holochain_core::{
    action::{Action, ActionWrapper},
    network::entry_with_header::EntryWithHeader,
    nucleus::ZomeFnCall,
    signal::{Signal, SignalReceiver},
};
use holochain_core_types::entry::Entry;
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

/// Possible messages used to influence the behavior of the CallBlockingTask
/// Currently the only action needed is to stop it, triggering its callback
pub enum ControlMsg {
    Stop,
}

/// A predicate function which examines an ActionWrapper to see if it is
/// the one it's looking for. `count` specifies how many of this Action to
/// look for before being satisfied.
struct CallFxCondition {
    count: usize,
    predicate: Box<Fn(&ActionWrapper) -> bool + 'static + Send>,
}

impl CallFxCondition {
    pub fn new(count: usize, predicate: Box<Fn(&ActionWrapper) -> bool + 'static + Send>) -> Self {
        Self { count, predicate }
    }

    /// If the predicate is satisfied, decrement the total number of checks
    pub fn run(&mut self, aw: &ActionWrapper) {
        if (self.predicate)(aw) {
            self.count -= 1;
        }
    }

    /// The true if the predicate returned `true` `count` times
    pub fn satisfied(&self) -> bool {
        self.count == 0
    }
}

/// A set of `CallFxCondition`s, each of which checks for a certain condition to be met
/// (usually for a certain action to be seen) a certain number of times.
/// When the condition specified is satisfied, it is removed from the set of checks.
///
/// When the set of checks goes from non-empty to empty, send a message via `tx`
/// to the `CallBlockingTask` on the other side
struct CallFxChecker {
    tx: ControlSender,
    conditions: Vec<CallFxCondition>,
}

impl CallFxChecker {
    pub fn new(tx: ControlSender) -> Self {
        Self {
            tx,
            conditions: Vec::new(),
        }
    }

    pub fn add<F>(&mut self, count: usize, f: F) -> ()
    where
        F: Fn(&ActionWrapper) -> bool + 'static + Send,
    {
        self.conditions
            .push(CallFxCondition::new(count, Box::new(f)));
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
        for condition in &mut self.conditions {
            condition.run(aw)
        }
        self.conditions.retain(|condition| !condition.satisfied());
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

/// A simple Task that blocks until it receives `ControlMsg::Stop`.
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
    current: Option<ZomeFnCall>,
    sender_rx: Receiver<ControlSender>,
    num_instances: usize,
}

impl Waiter {
    pub fn new(sender_rx: Receiver<ControlSender>, num_instances: usize) -> Self {
        Self {
            checkers: HashMap::new(),
            current: None,
            sender_rx,
            num_instances,
        }
    }

    /// Alter state based on signals that come in, if a checker is registered.
    /// A checker gets registered if a ControlSender was passed in from TestConductor.
    /// Some signals add a "condition", which is a function looking for other signals.
    /// When one of those "checkee" signals comes in, it removes the checker from the state.
    pub fn process_signal(&mut self, sig: Signal) {
        let num_instances = self.num_instances;
        match sig {
            Signal::Internal(ref aw) => {
                let aw = aw.clone();
                match (self.current_checker(), aw.action().clone()) {
                    // Pair every `ExecuteZomeFunction` with one `ReturnZomeFunctionResult`
                    (_, Action::ExecuteZomeFunction(call)) => match self.sender_rx.try_recv() {
                        Ok(sender) => {
                            self.add_call(call.clone(), sender);
                            self.current_checker().unwrap().add(1, move |aw| {
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
                    },

                    (Some(checker), Action::Commit((committed_entry, _))) => {
                        match committed_entry.clone() {
                            // Pair every `Commit` with N `Hold`s
                            Entry::App(_, _) => {
                                // TODO: is there a possiblity that this can get messed up if the same
                                // entry is committed multiple times?
                                checker.add(num_instances, move |aw| match aw.action() {
                                    Action::Hold(EntryWithHeader { entry, header: _ }) => {
                                        *entry == committed_entry
                                    }
                                    _ => false,
                                });
                            }
                            // Pair every `LinkAdd` with N `Hold`s and N `AddLink`s
                            Entry::LinkAdd(link_add) => {
                                checker.add(num_instances, move |aw| match aw.action() {
                                    Action::Hold(EntryWithHeader { entry, header: _ }) => {
                                        *entry == committed_entry
                                    }
                                    _ => false,
                                });
                                checker.add(num_instances, move |aw| {
                                    *aw.action() == Action::AddLink(link_add.clone().link().clone())
                                });
                            }
                            // Pair every `LinkRemove` with N `Hold`s
                            Entry::LinkRemove(_link_remove) => {
                                checker.add(num_instances, move |aw| match aw.action() {
                                    Action::Hold(EntryWithHeader { entry, header: _ }) => {
                                        *entry == committed_entry
                                    }
                                    _ => false,
                                });
                                println!("warn/waiter: LinkRemove not implemented!");
                            }
                            _ => (),
                        }
                    }

                    // Don't need to check for message stuff since hdk::send is blocking

                    // (Some(checker), Action::SendDirectMessage(data)) => {
                    //     let msg_id = data.msg_id;
                    //     match data.message {
                    //         DirectMessage::Custom(_) => {
                    //             checker.add(move |aw| {
                    //                 [
                    //                     Action::ResolveDirectConnection(msg_id.clone()),
                    //                     Action::SendDirectMessageTimeout(msg_id.clone()),
                    //                 ]
                    //                 .contains(aw.action())
                    //             });
                    //         }
                    //         _ => (),
                    //     }
                    // }

                    // Note that we ignore anything coming in if there's no active checker,
                    (None, _) => (),

                    // or if it's simply a signal we don't care about
                    _ => (),
                };

                self.run_checks(&aw);
            }

            _ => (),
        };
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

/// This Task is started with the TestConductor and is stopped with the TestConductor.
/// It runs in a Node worker thread, receiving Signals and running them through
/// the Waiter. Each TestConductor spawns its own MainBackgroundTask.
pub struct MainBackgroundTask {
    /// The Receiver<Signal> for the Conductor
    signal_rx: SignalReceiver,
    /// The Waiter is in a RefCell because perform() uses an immutable &self reference
    waiter: RefCell<Waiter>,
    /// This Mutex is flipped from true to false from within the TestConductor
    is_running: Arc<Mutex<bool>>,
}

impl MainBackgroundTask {
    pub fn new(
        signal_rx: SignalReceiver,
        sender_rx: Receiver<ControlSender>,
        is_running: Arc<Mutex<bool>>,
        num_instances: usize,
    ) -> Self {
        let this = Self {
            signal_rx,
            waiter: RefCell::new(Waiter::new(sender_rx, num_instances)),
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
            // rather than waiting for timeout, but it's more complicated and probably
            // involves adding some kind of control variant to the Signal enum
            match self.signal_rx.recv_timeout(Duration::from_millis(250)) {
                Ok(sig) => self.waiter.borrow_mut().process_signal(sig),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(err) => return Err(err.to_string()),
            }
        }

        for (_, checker) in self.waiter.borrow_mut().checkers.iter_mut() {
            println!("{}", "Shutting down lingering checker...".magenta().bold());
            checker.shutdown();
        }
        println!("Terminating MainBackgroundTask::perform() loop");
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsUndefined> {
        result.or_else(|e| {
            let error_string = cx.string(format!("unable to shut down background task: {}", e));
            cx.throw(error_string)
        })?;
        println!("MainBackgroundTask shut down");
        Ok(cx.undefined())
    }
}

#[cfg(test)]
mod tests {
    use super::{Action::*, *};
    use holochain_core::nucleus::ExecuteZomeFnResponse;
    use holochain_core_types::{
        chain_header::test_chain_header, entry::Entry, json::JsonString, link::link_add::LinkAdd,
    };
    use std::sync::mpsc::sync_channel;

    fn sig(a: Action) -> Signal {
        Signal::Internal(ActionWrapper::new(a))
    }

    fn mk_entry(ty: &'static str, content: &'static str) -> Entry {
        Entry::App(ty.into(), JsonString::from(content))
    }

    fn mk_entry_wh(entry: Entry) -> EntryWithHeader {
        EntryWithHeader {
            entry,
            header: test_chain_header(),
        }
    }

    // not needed as long as hdk::send is blocking
    // fn msg_data(msg_id: &str) -> DirectMessageData {
    //     DirectMessageData {
    //         address: "fake address".into(),
    //         message: DirectMessage::Custom(CustomDirectMessage {
    //             zome: "fake zome".into(),
    //             payload: Ok("fake payload".into()),
    //         }),
    //         msg_id: msg_id.into(),
    //         is_response: false,
    //     }
    // }

    fn zf_call(name: &str) -> ZomeFnCall {
        ZomeFnCall::new(name, None, name, "")
    }

    fn zf_response(call: ZomeFnCall) -> ExecuteZomeFnResponse {
        ExecuteZomeFnResponse::new(call, Ok(JsonString::from("")))
    }

    fn num_conditions(waiter: &Waiter, call: &ZomeFnCall) -> usize {
        waiter
            .checkers
            .get(&call)
            .expect("No checker for call")
            .conditions
            .len()
    }

    fn expect_final<F>(control_rx: Receiver<ControlMsg>, f: F)
    where
        F: FnOnce() -> (),
    {
        assert!(
            control_rx.try_recv().is_err(),
            "ControlMsg::Stop message received too early!"
        );
        f();
        assert!(
            control_rx.try_recv().is_ok(),
            "ControlMsg::Stop message not received!"
        );
    }

    fn test_waiter() -> (Waiter, SyncSender<ControlSender>) {
        let (sender_tx, sender_rx) = sync_channel(1);
        let waiter = Waiter::new(sender_rx, 1);
        (waiter, sender_tx)
    }

    /// Register a new callback, as if `callSync` were invoked
    fn test_register(sender_tx: &SyncSender<ControlSender>) -> Receiver<ControlMsg> {
        let (control_tx, control_rx) = sync_channel(1);
        sender_tx
            .send(control_tx)
            .expect("Could not send control sender");
        control_rx
    }

    #[test]
    fn can_await_commit_simple_ordering() {
        let (mut waiter, sender_tx) = test_waiter();
        let entry = mk_entry("t1", "x");
        let entry_wh = mk_entry_wh(entry.clone());
        let call = zf_call("c1");

        let control_rx = test_register(&sender_tx);
        assert_eq!(waiter.checkers.len(), 0);

        waiter.process_signal(sig(ExecuteZomeFunction(call.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        assert_eq!(num_conditions(&waiter, &call), 1);

        waiter.process_signal(sig(Commit((entry.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call), 2);

        waiter.process_signal(sig(Hold(entry_wh)));
        assert_eq!(num_conditions(&waiter, &call), 1);
        assert_eq!(waiter.checkers.len(), 1);

        expect_final(control_rx, || {
            waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call.clone()))))
        });
        assert_eq!(waiter.checkers.len(), 0);
    }

    #[test]
    fn can_await_commit_complex_ordering() {
        let (mut waiter, sender_tx) = test_waiter();
        let entry_1 = mk_entry("t1", "x");
        let entry_2 = mk_entry("t2", "y");
        let entry_1_wh = mk_entry_wh(entry_1.clone());
        let entry_2_wh = mk_entry_wh(entry_2.clone());
        let call = zf_call("c1");

        let control_rx = test_register(&sender_tx);
        assert_eq!(waiter.checkers.len(), 0);

        waiter.process_signal(sig(ExecuteZomeFunction(call.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        assert_eq!(num_conditions(&waiter, &call), 1);

        waiter.process_signal(sig(Commit((entry_1.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call), 2);

        waiter.process_signal(sig(Commit((entry_2.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call), 3);

        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call.clone()))));
        assert_eq!(num_conditions(&waiter, &call), 2);

        waiter.process_signal(sig(Hold(entry_2_wh.clone())));
        assert_eq!(num_conditions(&waiter, &call), 1);
        assert_eq!(waiter.checkers.len(), 1);

        expect_final(control_rx, || {
            waiter.process_signal(sig(Hold(entry_1_wh.clone())));
        });
        assert_eq!(waiter.checkers.len(), 0);
    }

    #[test]
    fn can_await_multiple_registered_zome_calls() {
        let (mut waiter, sender_tx) = test_waiter();
        let entry_1 = mk_entry("t1", "x");
        let entry_2 = mk_entry("t2", "y");
        let entry_3 = mk_entry("t3", "z");
        let entry_4 = mk_entry("t4", "w");
        let entry_1_wh = mk_entry_wh(entry_1.clone());
        let entry_2_wh = mk_entry_wh(entry_2.clone());
        let entry_3_wh = mk_entry_wh(entry_3.clone());
        let call_1 = zf_call("c1");
        let call_2 = zf_call("c2");
        let call_3 = zf_call("c3");

        // an "unregistered" zome call (not using `callSync` or `callWithPromise`)
        assert_eq!(waiter.checkers.len(), 0);
        waiter.process_signal(sig(ExecuteZomeFunction(call_1.clone())));
        assert_eq!(waiter.checkers.len(), 0);
        waiter.process_signal(sig(Commit((entry_1.clone(), None))));
        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_1.clone()))));
        assert_eq!(waiter.checkers.len(), 0);
        // no checkers should be registered during any of this

        // Now register a callback
        let control_rx_2 = test_register(&sender_tx);
        // which shouldn't change the checkers count yet
        assert_eq!(waiter.checkers.len(), 0);

        waiter.process_signal(sig(ExecuteZomeFunction(call_2.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        assert_eq!(num_conditions(&waiter, &call_2), 1);

        waiter.process_signal(sig(Commit((entry_2.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call_2), 2);

        waiter.process_signal(sig(Commit((entry_3.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call_2), 3);

        // a Hold left over from that first unregistered function: should do nothing
        waiter.process_signal(sig(Hold(entry_1_wh)));

        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_2.clone()))));
        assert_eq!(num_conditions(&waiter, &call_2), 2);

        // one more unregistered function call
        assert_eq!(waiter.checkers.len(), 1);
        waiter.process_signal(sig(ExecuteZomeFunction(call_3.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        waiter.process_signal(sig(Commit((entry_4.clone(), None))));
        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_3.clone()))));
        assert_eq!(waiter.checkers.len(), 1);
        // again, shouldn't change things at all

        waiter.process_signal(sig(Hold(entry_2_wh)));
        assert_eq!(num_conditions(&waiter, &call_2), 1);

        expect_final(control_rx_2, || {
            waiter.process_signal(sig(Hold(entry_3_wh)))
        });
        assert_eq!(waiter.checkers.len(), 0);

        // we don't even care that Hold(entry_4) was not seen,
        // we're done because it wasn't registered.
    }

    #[test]
    fn can_await_links() {
        let (mut waiter, sender_tx) = test_waiter();
        let call = zf_call("c1");
        let link_add = LinkAdd::new(
            &"base".to_string().into(),
            &"target".to_string().into(),
            "tag",
        );
        let entry = Entry::LinkAdd(link_add.clone());
        let entry_wh = mk_entry_wh(entry.clone());

        let control_rx = test_register(&sender_tx);
        assert_eq!(waiter.checkers.len(), 0);

        waiter.process_signal(sig(ExecuteZomeFunction(call.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        assert_eq!(num_conditions(&waiter, &call), 1);

        // this adds two actions to await
        waiter.process_signal(sig(Commit((entry.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call), 3);

        waiter.process_signal(sig(Hold(entry_wh)));
        assert_eq!(num_conditions(&waiter, &call), 2);

        waiter.process_signal(sig(AddLink(link_add.link().clone())));
        assert_eq!(num_conditions(&waiter, &call), 1);
        assert_eq!(waiter.checkers.len(), 1);

        expect_final(control_rx, || {
            waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call.clone()))))
        });
        assert_eq!(waiter.checkers.len(), 0);
    }

    #[test]
    fn can_await_registered_and_unregistered_zome_calls() {
        let (mut waiter, sender_tx) = test_waiter();
        let entry_1 = mk_entry("t1", "x");
        let entry_2 = mk_entry("t2", "y");
        let entry_3 = mk_entry("t3", "z");
        let entry_1_wh = mk_entry_wh(entry_1.clone());
        let entry_2_wh = mk_entry_wh(entry_2.clone());
        let entry_3_wh = mk_entry_wh(entry_3.clone());
        let call_1 = zf_call("c1");
        let call_2 = zf_call("c2");

        let control_rx_1 = test_register(&sender_tx);
        assert_eq!(waiter.checkers.len(), 0);

        waiter.process_signal(sig(ExecuteZomeFunction(call_1.clone())));
        assert_eq!(waiter.checkers.len(), 1);
        assert_eq!(num_conditions(&waiter, &call_1), 1);

        waiter.process_signal(sig(Commit((entry_1.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call_1), 2);

        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_1.clone()))));
        assert_eq!(num_conditions(&waiter, &call_1), 1);

        // register a second callback
        let control_rx_2 = test_register(&sender_tx);
        // which shouldn't change the checkers count yet
        assert_eq!(waiter.checkers.len(), 1);

        waiter.process_signal(sig(ExecuteZomeFunction(call_2.clone())));
        assert_eq!(waiter.checkers.len(), 2);
        assert_eq!(num_conditions(&waiter, &call_2), 1);

        waiter.process_signal(sig(Commit((entry_2.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call_2), 2);

        waiter.process_signal(sig(Commit((entry_3.clone(), None))));
        assert_eq!(num_conditions(&waiter, &call_2), 3);

        expect_final(control_rx_1, || {
            waiter.process_signal(sig(Hold(entry_1_wh)));
        });

        waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_2.clone()))));
        assert_eq!(num_conditions(&waiter, &call_2), 2);

        waiter.process_signal(sig(Hold(entry_2_wh)));
        assert_eq!(num_conditions(&waiter, &call_2), 1);

        expect_final(control_rx_2, || {
            waiter.process_signal(sig(Hold(entry_3_wh)))
        });
        assert_eq!(waiter.checkers.len(), 0);
    }

    // not needed as long as hdk::send is blocking
    // #[test]
    // fn can_await_direct_messages() {
    //     let (mut waiter, sender_tx) = test_waiter();
    //     let _entry_1 = mk_entry("a", "x");
    //     let _entry_2 = mk_entry("b", "y");
    //     let _entry_3 = mk_entry("c", "z");
    //     let call_1 = zf_call("1");
    //     let call_2 = zf_call("2");
    //     let msg_id_1 = "m1";
    //     let msg_id_2 = "m2";

    //     let control_rx_1 = test_register(&sender_tx);
    //     waiter.process_signal(sig(ExecuteZomeFunction(call_1.clone())));
    //     assert_eq!(num_conditions(&waiter, &call_1), 1);

    //     waiter.process_signal(sig(SendDirectMessage(msg_data(msg_id_1))));
    //     assert_eq!(num_conditions(&waiter, &call_1), 2);

    //     waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_1.clone()))));
    //     assert_eq!(num_conditions(&waiter, &call_1), 1);

    //     let control_rx_2 = test_register(&sender_tx);
    //     waiter.process_signal(sig(ExecuteZomeFunction(call_2.clone())));
    //     assert_eq!(num_conditions(&waiter, &call_2), 1);

    //     waiter.process_signal(sig(SendDirectMessage(msg_data(msg_id_2))));
    //     assert_eq!(num_conditions(&waiter, &call_2), 2);

    //     waiter.process_signal(sig(ReturnZomeFunctionResult(zf_response(call_2.clone()))));
    //     assert_eq!(num_conditions(&waiter, &call_2), 1);

    //     expect_final(control_rx_1, || {
    //         waiter.process_signal(sig(ResolveDirectConnection(msg_id_1.to_string())));
    //     });
    //     expect_final(control_rx_2, || {
    //         waiter.process_signal(sig(SendDirectMessageTimeout(msg_id_2.to_string())));
    //     });
    // }
}
