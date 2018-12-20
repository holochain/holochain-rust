use holochain_container_api::{
    context_builder::ContextBuilder,
    Holochain,
};
use holochain_core_types::{
    cas::content::Address,
    dna::{Dna, capabilities::CapabilityCall},
    agent::AgentId,
    json::JsonString
};
use neon::context::Context;
use neon::prelude::*;
use std::sync::Arc;
use std::convert::TryFrom;

pub struct App {
    instance: Holochain,
}

declare_types! {
    pub class JsApp for App {
        init(mut ctx) {
            let agent_name = ctx.argument::<JsString>(0)?.to_string(&mut ctx)?.value();
            let dna_data = ctx.argument::<JsString>(1)?.to_string(&mut ctx)?.value();

            let agent = AgentId::generate_fake(&agent_name);
            let context = ContextBuilder::new().with_agent(agent).spawn();
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
                let call =  Some(CapabilityCall::new(
                    cap.to_string(),
                    Address::from(""), //FIXME
                    None,
                ));
                app.instance.call(&zome, call, &fn_name, &params)
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

register_module!(mut ctx, { ctx.export_class::<JsApp>("HolochainApp") });
