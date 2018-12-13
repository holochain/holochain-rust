use holochain_container_api::container::Container;

use neon::{context::Context, prelude::*};

declare_types! {

    pub class JsScenarioContainer for ScenarioContainer {

        init(mut cx) {
            let tempdir = tempdir().unwrap();
            let agent_name = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let dna_data = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();

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
                    let error_string = cx.string(format!("Unable to instantiate DNA with error: {}", error));
                    cx.throw(error_string)
                })?,
            })
        }

        method start(mut cx) {
            let mut this = cx.this();

            let start_result = {
                let guard = cx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.start()
            };

            start_result.or_else(|_| {
                let error_string = cx.string("unable to start hApp");
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method stop(mut cx) {
            let mut this = cx.this();

            let start_result = {
                let guard = cx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.stop()
            };

            start_result.or_else(|_| {
                let error_string = cx.string("unable to stop hApp");
                cx.throw(error_string)
            })?;

            Ok(cx.undefined().upcast())
        }

        method call(mut cx) {
            let zome = cx.argument::<JsString>(0)?.to_string(&mut cx)?.value();
            let cap = cx.argument::<JsString>(1)?.to_string(&mut cx)?.value();
            let fn_name = cx.argument::<JsString>(2)?.to_string(&mut cx)?.value();
            let params = cx.argument::<JsString>(3)?.to_string(&mut cx)?.value();
            let mut this = cx.this();

            let call_result = {
                let guard = cx.lock();
                let mut app = this.borrow_mut(&guard);

                app.instance.call(&zome, &cap, &fn_name, &params)
            };

            let res_string = call_result.or_else(|e| {
                let error_string = cx.string(format!("unable to call zome function: {:?}", &e));
                cx.throw(error_string)
            })?;

            let result_string: String = res_string.into();
            Ok(cx.string(result_string).upcast())
        }
    }