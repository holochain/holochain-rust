use holochain_core::{
    context::Context as HolochainContext, logger::Logger, persister::SimplePersister,
};
use holochain_cas_implementations::{
    cas::file::FilesystemStorage,
    cas::memory::MemoryStorage,
    eav::memory::EavMemoryStorage
};
use holochain_container_api::Holochain;
use holochain_net::p2p_network::P2pNetwork;
use holochain_core_types::{
    dna::Dna,
    agent::Agent,
    json::JsonString
};
use neon::context::Context;
use neon::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use tempfile::tempdir;
use std::convert::TryFrom;

#[derive(Clone, Debug)]
struct NullLogger {}

impl Logger for NullLogger {
    fn log(&mut self, _msg: String) {}
}

pub struct App {
    instance: Holochain,
    hash: String,
}

declare_types! {
    pub class JsApp for App {
        init(mut ctx) {
            let tempdir = tempdir().unwrap();
            let agent_name = ctx.argument::<JsString>(0)?.to_string(&mut ctx)?.value();
            let dna_data = ctx.argument::<JsString>(1)?.to_string(&mut ctx)?.value();

            let agent = Agent::generate_fake(&agent_name);
            let file_storage = Arc::new(RwLock::new(
                FilesystemStorage::new(tempdir.path().to_str().unwrap()).unwrap(),
            ));
            let mock_net = Arc::new(Mutex::new(P2pNetwork::new(
                Box::new(|_r| Ok(())),
                &json!({
                    "backend": "mock"
                }).into(),
            ).unwrap()));

            let context = HolochainContext::new(
                agent,
                Arc::new(Mutex::new(NullLogger {})),
                Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
                Arc::new(RwLock::new(MemoryStorage::new().unwrap())),
                Arc::new(RwLock::new(EavMemoryStorage::new().unwrap())),
                mock_net,
            ).unwrap();

            let dna = Dna::try_from(JsonString::from(dna_data)).expect("unable to parse dna data");

            Ok(App {
                instance: Holochain::new(dna, Arc::new(context))
                .or_else(|error| {
                    let error_string = ctx.string(format!("Unable to instantiate DNA with error: {}", error));
                    ctx.throw(error_string)
                })?,
                hash: "ab83bae71f53b18d7ea8db36193baf48bf82aff392aab4".into(),
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

            let res_string = call_result.or_else(|_| {
                let error_string = ctx.string("unable to call zome function");
                ctx.throw(error_string)
            })?;

            let result_string: String = res_string.into();
            Ok(ctx.string(result_string).upcast())
        }
    }
}

register_module!(mut ctx, { ctx.export_class::<JsApp>("HolochainApp") });
