use neon::{context::Context, prelude::*};
use std::{
    collections::HashSet,
    sync::{
        mpsc::{sync_channel, SyncSender},
        Arc, Mutex,
    },
};

use holochain_conductor_api::{
    config::{load_configuration, Configuration},
    conductor::Conductor as RustConductor,
};
use holochain_core::{
    action::Action,
    signal::{signal_channel, Signal, SignalReceiver},
};
use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    dna::capabilities::CapabilityCall,
    entry::Entry,
};
use holochain_node_test_waiter::waiter::{CallBlockingTask, ControlMsg, MainBackgroundTask};

/// Block until Hold(agent.public_address) is seen for each agent in the conductor.
/// NOTE that this consumes a bunch of signals related to initialization!
/// The `Waiter` currently doesn't care about these, but beware.
fn await_held_agent_ids(config: Configuration, signal_rx: &SignalReceiver) {
    let mut agent_addresses: HashSet<String> = config
        .agents
        .iter()
        .map(|c| c.public_address.to_string())
        .collect();
    loop {
        if let Signal::Internal(aw) = signal_rx.recv().unwrap() {
            let action = aw.action();
            if let Action::Hold(Entry::AgentId(id)) = action {
                agent_addresses.remove(&id.key);
            }
            if agent_addresses.is_empty() {
                break;
            }
        }
    }
}

pub struct TestConductor {
    conductor: RustConductor,
    sender_tx: Option<SyncSender<SyncSender<ControlMsg>>>,
    is_running: Arc<Mutex<bool>>,
    is_started: bool,
}

declare_types! {

    // A TestConductor can be initialized either by:
    // - an Object representation of a Configuration struct
    // - a string representing TOML
    pub class JsTestConductor for TestConductor {
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
            let conductor = RustConductor::from_config(config);
            let is_running = Arc::new(Mutex::new(false));

            Ok(TestConductor { conductor, sender_tx: None, is_running, is_started: false })
        }

        // Start the backing Conductor and spawn a MainBackgroundTask
        // Accepts a callback which will be passed on to MainBackgroundTask
        // and called when that exits. This callback is used to fuel a Promise
        // when calling .start()
        method start(mut cx) {
            let js_callback: Handle<JsFunction> = cx.argument(0)?;
            let mut this = cx.this();

            let (signal_tx, signal_rx) = signal_channel();
            let (sender_tx, sender_rx) = sync_channel(1);

            let result = {
                let guard = cx.lock();
                let tc = &mut *this.borrow_mut(&guard);
                tc.sender_tx = Some(sender_tx);
                {
                    let mut is_running = tc.is_running.lock().unwrap();
                    *is_running = true;
                }
                tc.conductor.load_config_with_signal(Some(signal_tx)).and_then(|_| {
                    tc.conductor.start_all_instances().map_err(|e| e.to_string()).map(|_| {
                        await_held_agent_ids(tc.conductor.config(), &signal_rx);
                        let num_instances = tc.conductor.instances().len();
                        let background_task = MainBackgroundTask::new(signal_rx, sender_rx, tc.is_running.clone(), num_instances);
                        background_task.schedule(js_callback);
                        tc.is_started = true;

                    })
                })
            };

            result.or_else(|e| {
                cx.throw_error(format!("unable to start conductor: {}", e))
            }).map(|_| {
                cx.boolean(true).upcast()
            })
        }

        // Stop the backing conductor and break the listening loop in the MainBackgroundTask
        method stop(mut cx) {
            let mut this = cx.this();

            let stop_result: Result<(), String> = {
                let guard = cx.lock();
                let tc = &mut *this.borrow_mut(&guard);

                let mut is_running = tc.is_running.lock().unwrap();
                // This causes MainBackgroundTask to eventually terminate
                *is_running = false;

                // TODO are we sure shutdown should not return a Result?
                let result = tc.conductor.shutdown();
                Ok(result)
            };

            stop_result.or_else(|e| {
                let error_string = cx.string(format!("unable to stop conductor: {}", e));
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method call(mut cx) {
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let zome = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();
            let fn_name = cx.argument::<JsString>(2)?.to_string(&mut cx)?.value();
            let params = cx.argument::<JsString>(3)?.to_string(&mut cx)?.value();

            let mut this = cx.this();

            let call_result = {
                let guard = cx.lock();
                let tc = &mut *this.borrow_mut(&guard);
                if !tc.is_started {
                    panic!("TestConductor: cannot use call() before start()");
                }
                let cap = Some(CapabilityCall::new(
                    Address::from(""), //FIXME
                    None,
                ));
                let instance_arc = tc.conductor.instances().get(&instance_id)
                    .expect(&format!("No instance with id: {}", instance_id));
                let mut instance = instance_arc.write().unwrap();
                instance.call(&zome, cap, &fn_name, &params)
            };

            let res_string = call_result.or_else(|e| {
                let error_string = cx.string(format!("unable to call zome function: {:?}", &e));
                cx.throw(error_string)
            })?;

            let result_string: String = res_string.into();

            Ok(cx.string(result_string).upcast())
        }

        // This sets up the state of MainBackgroundTask to listen for the next ExecuteZomeFunction
        // action and does its magic of observing incoming actions to invoke the callback once it
        // determines that all test-relevant network activity has completed
        method register_callback(mut cx) {
            let js_callback: Handle<JsFunction> = cx.argument(0)?;
            let this = cx.this();
            {
                let guard = cx.lock();
                let tc = &*this.borrow(&guard);

                if !tc.is_started {
                    panic!("TestConductor: cannot use register_callback() before start()");
                }

                let (tx, rx) = sync_channel(0);
                let task = CallBlockingTask { rx };
                task.schedule(js_callback);
                tc
                    .sender_tx
                    .as_ref()
                    .expect("Conductor sender channel not initialized")
                    .send(tx)
                    .expect("Could not send to sender channel");
            }
            Ok(cx.undefined().upcast())
        }

        // Fetch the agent address from within the instance
        method agent_id(mut cx) {
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let this = cx.this();
            let result = {
                let guard = cx.lock();
                let tc = this.borrow(&guard);

                if !tc.is_started {
                    panic!("TestConductor: cannot use agent_id() before start()");
                }
                let instance = tc.conductor.instances().get(&instance_id)
                    .expect(&format!("No instance with id: {}", instance_id))
                    .read().unwrap();
                let out = instance.context().state().ok_or("No state?".to_string())
                    .and_then(|state| state
                        .agent().get_agent_address()
                        .map_err(|e| e.to_string()));
                out
            };

            let hash = result.or_else(|e: String| {
                let error_string = cx.string(format!("unable to call zome function: {:?}", &e));
                cx.throw(error_string)
            })?;
            Ok(cx.string(hash.to_string()).upcast())
        }

        // Fetch the DNA address from within the instance
        method dna_address(mut cx) {
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let this = cx.this();
            let maybe_dna = {
                let guard = cx.lock();
                let tc = this.borrow(&guard);

                if !tc.is_started {
                    panic!("TestConductor: cannot use dna_address() before start()");
                }
                let instance = tc.conductor.instances().get(&instance_id)
                    .expect(&format!("No instance with id: {}", instance_id))
                    .read().unwrap();
                let out = instance.context().state().ok_or("No state?".to_string())
                    .and_then(|state| state
                        .nucleus()
                        .dna
                        .clone()
                        .ok_or(String::from("No DNA set in instance state"))
                    );
                out
            };

            let dna = maybe_dna.or_else(|e: String| {
                let error_string = cx.string(format!("unable to get DNA: {:?}", &e));
                cx.throw(error_string)
            })?;
            let address = dna.address();
            Ok(cx.string(address.to_string()).upcast())
        }
    }
}
