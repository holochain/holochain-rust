use holochain_cas_implementations::{
    cas::{file::FilesystemStorage, memory::MemoryStorage},
    eav::memory::EavMemoryStorage,
};
use holochain_container_api::Holochain;
use holochain_core::{
    context::{mock_network_config, Context as HolochainContext},
    logger::Logger,
    persister::SimplePersister,
};
use holochain_core_types::{agent::AgentId, dna::Dna, json::JsonString};
use neon::{context::Context, prelude::*};
use std::{
    convert::TryFrom,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use tempfile::tempdir;

use crate::config::*;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

pub struct App {
    instance: Holochain,
}

pub struct HcTest {}

declare_types! {

    pub class JsHcTest for HcTest {

        init(mut cx) {
            Ok(HcTest {})
        }

        method agent(mut cx) {
            let name = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let obj = AgentData { name };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method dna(mut cx) {
            let path = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let path = PathBuf::from(path);
            let obj = DnaData { path };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method instance(mut cx) {
            let agent = AgentData { name: "test-agent".into() };
            let dna = DnaData { path: PathBuf::from("test-dna") };
            let obj = InstanceData { agent, dna };
            Ok(neon_serde::to_value(&mut cx, &obj)?)
        }

        method scenario(mut cx) {
            let mut i = 0;
            // let mut instances = Vec::new();
            while let Some(arg) = cx.argument_opt(i) {
                println!("got some args to handle");
                i += 1;
            };
            Ok(neon::types::JsNull::new().as_value(&mut cx))
        }
    }

    pub class JsScenarioConfig for ScenarioConfig {
        init(mut cx) {
            let instances = cx.argument::<JsArray>(0)?
                .to_vec(&mut cx)?
                .into_iter()
                .map(|v| neon_serde::from_value(&mut cx, v).expect("scenario() argument deserialization failed"))
                .collect();
            Ok(ScenarioConfig(instances))
        }

        // method run(mut cx) {
        //     let mut this = ctx.this();
        //     let func = cx.argument::<JsFunction>(0)?;
        //     func.call(&mut cx, this, args);
        // }
    }

    pub class JsApp for App {
        init(mut ctx) {
            let tempdir = tempdir().unwrap();
            let agent_name = ctx.argument::<JsString>(0)?.to_string(&mut ctx)?.value();
            let dna_data = ctx.argument::<JsString>(1)?.to_string(&mut ctx)?.value();

            let agent = AgentId::generate_fake(&agent_name);
            let file_storage = Arc::new(RwLock::new(
                FilesystemStorage::new(tempdir.path().to_str().unwrap()).unwrap(),
            ));

            let context = HolochainContext::new(
                agent,
                Arc::new(Mutex::new(NullLogger {})),
                Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
                Arc::new(RwLock::new(MemoryStorage::new())),
                Arc::new(RwLock::new(EavMemoryStorage::new())),
                mock_network_config(),
            ).unwrap();
            let dna = Dna::try_from(JsonString::from(dna_data)).expect("unable to parse dna data");
            Ok(App {
                instance: Holochain::new(dna, Arc::new(context))
                .or_else(|error| {
                    let error_string = ctx.string(format!("Unable to instantiate DNA with error: {}", error));
                    ctx.throw(error_string)
                })?,
            })
        }

        method start(mut ctx) {
            let mut this = ctx.this();

            let start_result = {
                let guard = ctx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.start()
            };

            start_result.or_else(|_| {
                let error_string = ctx.string("unable to start hApp");
                ctx.throw(error_string)
            })?;

            Ok(ctx.undefined().upcast())
        }

        method stop(mut ctx) {
            let mut this = ctx.this();

            let start_result = {
                let guard = ctx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.stop()
            };

            start_result.or_else(|_| {
                let error_string = ctx.string("unable to stop hApp");
                ctx.throw(error_string)
            })?;

            Ok(ctx.undefined().upcast())
        }

        method call(mut ctx) {
            let zome = ctx.argument::<JsString>(0)?.to_string(&mut ctx)?.value();
            let cap = ctx.argument::<JsString>(1)?.to_string(&mut ctx)?.value();
            let fn_name = ctx.argument::<JsString>(2)?.to_string(&mut ctx)?.value();
            let params = ctx.argument::<JsString>(3)?.to_string(&mut ctx)?.value();
            let mut this = ctx.this();

            let call_result = {
                let guard = ctx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.call(&zome, &cap, &fn_name, &params)
            };

            let res_string = call_result.or_else(|e| {
                let error_string = ctx.string(format!("unable to call zome function: {:?}", &e));
                ctx.throw(error_string)
            })?;

            let result_string: String = res_string.into();
            Ok(ctx.string(result_string).upcast())
        }
    }
}

register_module!(mut ctx, {
    ctx.export_class::<JsApp>("HolochainApp")?;
    ctx.export_class::<JsHcTest>("HcTest")?;
    Ok(())
});
