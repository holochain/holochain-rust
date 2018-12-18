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

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

type JsCallback<'a> = Handle<'a, JsFunction>;

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

    pub fn add(&mut self, f: F) -> ()
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

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
        result.map(|_| cx.undefined())
    }
}

/// A singleton which runs in a Task and is the receiver for the Signal channel.
/// - handles incoming `ZomeFnCall`s, attaching and activating a new `CallFxChecker`
/// - handles incoming Signals, running all `CallFxChecker` closures
struct Waiter<'a> {
    checkers: HashMap<ZomeFnCall, CallFxChecker>,
    current: Option<ZomeFnCall>, // maybe RefCell
    callback_rx: CallbackReceiver<'a>,
}

type CallbackReceiver<'a> = Receiver<JsCallback<'a>>;

impl Waiter<'_> {
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
                match aw.action {
                    Action::ExecuteZomeFn(call) => {
                        let callback = self.callback_rx.recv();
                        self.add_call(call, callback);
                    }
                    Action::Commit((entry, _)) => match entry {
                        App(_, _) => self
                            .current_checker()
                            .unwrap()
                            .add(|aw| aw.action == Action::Hold(entry)),
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

    fn add_call(&mut self, call: ZomeFnCall, callback: JsCallback<'_>) {
        self.checkers.insert(call, CallFxChecker::new(callback));
        self.current = Some(call);
    }
}

enum WaiterMsg {
    Stop,
}

struct HabitatSignalTask<'a> {
    signal_rx: SignalReceiver,
    waiter: RefCell<Waiter<'a>>,
}

impl HabitatSignalTask<'_> {
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

pub struct Habitat {
    container: Container,
    callback_tx: SyncSender<JsCallback>,
}

fn signal_callback(mut cx: FunctionContext) -> JsResult<JsNull> {
    panic!("never should happen");
    Ok(cx.null())
}

fn promise_callback(mut cx: FunctionContext) -> JsResult<JsNull> {
    panic!("callback called!!");
    Ok(cx.null())
}

declare_types! {

    /// A Habitat can be initialized either by:
    /// - an Object representation of a Configuration struct
    /// - a string representing TOML
    pub class JsHabitat for Habitat {
        init(mut cx) {
            let config_arg: Handle<JsValue> = cx.argument(0)?;
            let config: Configuration = if config_arg.is_a::<JsObject>() {
                neon_serde::from_value(&mut cx, config_arg)?
            } else if config_arg.is_a::<JsString>() {
                let toml_str: String = neon_serde::from_value(&mut cx, config_arg)?;
                load_configuration(&toml_str).expect("Could not load TOML config")
            } else {
                panic!("Invalid type specified for config, must be object or string");
            };
            let (signal_tx, signal_rx) = signal_channel();
            let container = Container::from_config(config).with_signal_channel(signal_tx);

            let result = {
                let js_callback = JsFunction::new(&mut cx, signal_callback).unwrap();
                let mut this = cx.this();

                let result: Result<_, String> = {
                    let guard = cx.lock();
                    let hab = &mut *this.borrow_mut(&guard);
                    let (signal_tx, signal_rx) = signal_channel();
                    let (callback_tx, callback_rx) = sync_channel(100);
                    hab.container.load_config_with_signal(Some(signal_tx))?;

                    let waiter = Waiter::new(callback_rx);
                    let signal_task = HabitatSignalTask::new(signal_rx, callback_rx);
                    signal_task.schedule(js_callback);
                    hab.container.start_all_instances().map_err(|e| e.to_string()).and_then(|_| {
                        Ok(callback_tx)
                    })
                };
                result
            };

            let callback_tx = result.or_else(|e| {
                let error_string = cx.string(format!("unable to initialize habitat: {}", e));
                cx.throw(error_string)
            })?;

            Ok(Habitat { container, callback_tx })
        }

        method start(mut cx) {
            let js_callback = JsFunction::new(&mut cx, signal_callback).unwrap();
            let mut this = cx.this();

            let start_result: Result<(), String> = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                hab.container.start_all_instances().map_err(|e| e.to_string())
            };

            start_result.or_else(|e| {
                let error_string = cx.string(format!("unable to start habitat: {}", e));
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method stop(mut cx) {
            let mut this = cx.this();

            let stop_result: Result<(), String> = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                hab.container.stop_all_instances().map_err(|e| e.to_string())
            };

            stop_result.or_else(|e| {
                let error_string = cx.string(format!("unable to stop habitat: {}", e));
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method call(mut cx) {
            let js_callback = JsFunction::new(&mut cx, signal_callback).unwrap();
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let zome = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();
            let cap = cx.argument::<JsString>(2)?.to_string(&mut cx)?.value();
            let fn_name = cx.argument::<JsString>(3)?.to_string(&mut cx)?.value();
            let params = cx.argument::<JsString>(4)?.to_string(&mut cx)?.value();
            let mut this = cx.this();

            let call_result = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                let instance_arc = hab.container.instances().get(&instance_id)
                    .expect(&format!("No instance with id: {}", instance_id));
                let mut instance = instance_arc.write().unwrap();
                let val = instance.call(&zome, &cap, &fn_name, &params);
                let (tx, rx) = sync_channel(0);
                let task = SignalWaiterTask {rx};
                task.schedule(js_callback);
                val
            };

            let res_string = call_result.or_else(|e| {
                let error_string = cx.string(format!("unable to call zome function: {:?}", &e));
                cx.throw(error_string)
            })?;

            let result_string: String = res_string.into();

            // let completion_callback =
            Ok(cx.string(result_string).upcast())
        }
    }
}

// impl Task for HabitatSignalTask {
//     type Output = ();
//     type Error = String;
//     type JsEvent = JsNumber;

//     fn perform(&self) -> Result<(), String> {
//         use std::io::{self, Write};
//         while let Ok(sig) = self.signal_rx.recv() {
//             print!(".");
//             io::stdout().flush().unwrap();
//         }
//         Ok(())
//     }

//     fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
//         Ok(cx.number(17))
//     }
// }

// struct SignalWaiterTask {
//     rx: Receiver<WaiterMsg>,
// }

// impl Task for SignalWaiterTask {
//     type Output = ();
//     type Error = String;
//     type JsEvent = JsUndefined;

//     fn perform(&self) -> Result<(), String> {
//         while let Ok(sig) = self.rx.recv() {
//             match sig {
//                 WaiterMsg::Stop => break
//             }
//         }
//         Ok(())
//     }

//     fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
//         result.map(|_| cx.undefined())
//     }
// }

register_module!(mut cx, {
    cx.export_class::<JsHabitat>("Habitat")?;
    cx.export_class::<JsConfigBuilder>("ConfigBuilder")?;
    Ok(())
});
