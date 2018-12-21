use holochain_container_api::{
    config::{load_configuration, Configuration},
    container::{Container as RustContainer},
};
use holochain_core::{
    logger::Logger,
    signal::{signal_channel, SignalReceiver},
};
use holochain_core_types::{
    cas::content::Address,
    dna::{capabilities::CapabilityCall},
};
use neon::{context::Context, prelude::*};

use crate::config::*;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

pub struct NodeContainer {
    container: RustContainer,
    _signal_rx: SignalReceiver,
}

declare_types! {

    /// A Container can be initialized either by:
    /// - an Object representation of a Configuration struct
    /// - a string representing TOML
    pub class JsContainer for NodeContainer {
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
            let (signal_tx, _signal_rx) = signal_channel();
            let container = RustContainer::from_config(config).with_signal_channel(signal_tx);
            Ok(NodeContainer { container, _signal_rx })
        }

        method start(mut cx) {
            let mut this = cx.this();

            let start_result: Result<(), String> = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                hab.container.load_config().and_then(|_| {
                    hab.container.start_all_instances().map_err(|e| e.to_string())
                })
            };

            start_result.or_else(|e| {
                let error_string = cx.string(format!("unable to start container: {}", e));
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
                let error_string = cx.string(format!("unable to stop container: {}", e));
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method call(mut cx) {
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let zome = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();
            let cap_name = cx.argument::<JsString>(2)?.to_string(&mut cx)?.value();
            let fn_name = cx.argument::<JsString>(3)?.to_string(&mut cx)?.value();
            let params = cx.argument::<JsString>(4)?.to_string(&mut cx)?.value();
            let mut this = cx.this();

            let call_result = {
                let guard = cx.lock();
                let hab = &mut *this.borrow_mut(&guard);
                let cap = Some(CapabilityCall::new(
                    cap_name.to_string(),
                    Address::from(""), //FIXME
                    None,
                ));
                let instance_arc = hab.container.instances().get(&instance_id)
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

        method agent_id(mut cx) {
            let instance_id = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let this = cx.this();
            let result = {
                let guard = cx.lock();
                let hab = this.borrow(&guard);
                let instance = hab.container.instances().get(&instance_id)
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
    }
}

register_module!(mut cx, {
    cx.export_class::<JsContainer>("Container")?;
    cx.export_class::<JsConfigBuilder>("ConfigBuilder")?;
    Ok(())
});
