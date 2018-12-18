use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::memory::EavMemoryStorage,
};
use holochain_container_api::{
    config::{
        AgentConfiguration, Configuration, DnaConfiguration, InstanceConfiguration,
        LoggerConfiguration, StorageConfiguration, load_configuration,
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
use std::{
    convert::TryFrom,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    thread,
};
use snowflake::ProcessUniqueId;
use tempfile::tempdir;

use crate::config::*;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

pub struct Habitat {
    container: Container,
    // signal_task: HabitatSignalTask,
}

fn signal_callback(mut cx: FunctionContext) -> JsResult<JsNull> {
    panic!("never should happen");
    Ok(cx.null())
}

fn promise_callback(mut cx: FunctionContext) -> JsResult<JsNull> {
    panic!("callback called");
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
            Ok(Habitat { container })
        }

        method start(mut cx) {
            let js_callback = JsFunction::new(&mut cx, signal_callback).unwrap();
            let mut this = cx.this();

            let start_result: Result<(), String> = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                let (signal_tx, signal_rx) = signal_channel();
                hab.container.load_config_with_signal(Some(signal_tx)).and_then(|_| {
                    let signal_task = HabitatSignalTask {signal_rx};
                    signal_task.schedule(js_callback);
                    hab.container.start_all_instances().map_err(|e| e.to_string())
                })
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

            let completion_callback = 
            Ok(cx.string(result_string).upcast())
        }
    }
}

/// Encapsulates the side-effects of running a particular signal which need to be waited for
enum WaiterMsg {
    Stop
}

struct WaiterComm {
    predicate: Box<Fn(Signal) -> bool>,
    tx: SyncSender<WaiterMsg>,
}

type WaiterPool = Arc<RwLock<ProcessUniqueId, WaiterComm>>;

struct HabitatSignalTask {
    signal_rx: SignalReceiver,
    waiter_pool: WaiterPool,
}

impl HabitatSignalTask {
    pub fn new(signal_rx: SignalReceiver) -> Self {
        let this = Self { signal_rx };
        this
    }

    fn process_incoming_signal(&self, sig: Signal) {
        match sig {
            Signal::Internal(action_wrapper) => {
                match action_wrapper.action {
                    Action::Commit((entry, _)) => {
                        match entry {
                            App(_, _) => {

                            }
                        }
                    },
                    _ => ()
                }
            }
        }
    }

    fn process_outgoing(&self) {
        self.waiter_pool.write().unwrap().remove(self.action_id)
    }

    fn add_waiter(&self, action_id: ProcessUniqueId, action: Action) {
        self.waiter_pool.write().unwrap().insert(id, )
    }

    fn remove_waiter(&self, id: ProcessUniqueId) {
        
    }

    fn check(&self) {

    }
}

impl Task for HabitatSignalTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsNumber;

    fn perform(&self) -> Result<(), String> {
        use std::io::{self, Write};
        while let Ok(sig) = self.signal_rx.recv() {
            print!(".");
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
        Ok(cx.number(17))
    }
}

struct SignalWaiterTask {
    rx: Receiver<WaiterMsg>,
}

impl Task for SignalWaiterTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        while let Ok(sig) = self.rx.recv() {
            match sig {
                WaiterMsg::Stop => break
            }
        }
        Ok(())
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsNumber> {
        result.map(|_| cx.undefined())
    }
}


register_module!(mut cx, {
    cx.export_class::<JsHabitat>("Habitat")?;
    cx.export_class::<JsConfigBuilder>("ConfigBuilder")?;
    Ok(())
});
