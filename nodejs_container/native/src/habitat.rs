use holochain_container_api::{
    config::{load_configuration, Configuration},
    container::Container,
};
use holochain_core::{
    action::Action,
    signal::{signal_channel, SignalReceiver},
};
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

use crate::{
    config::{ConfigBuilder, JsConfigBuilder},
    waiter::{HabitatSignalTask, JsCallback, Waiter},
};

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
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
            let container = Container::from_config(config);

            let result = {
                let js_callback = JsFunction::new(&mut cx, signal_callback).unwrap();
                let (signal_tx, signal_rx) = signal_channel();
                let (callback_tx, callback_rx) = sync_channel(100);
                let waiter = Waiter::new(callback_rx);
                let signal_task = HabitatSignalTask::new(signal_rx, callback_rx);
                signal_task.schedule(js_callback);

                container.load_config_with_signal(Some(signal_tx)).map(|_| callback_tx)
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
