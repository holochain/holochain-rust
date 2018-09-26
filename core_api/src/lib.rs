//! holochain_core_api provides a library for container applications to instantiate and run holochain applications.
//!
//! # Examples
//!
//! ``` rust
//! extern crate holochain_core;
//! extern crate holochain_core_api;
//! extern crate holochain_dna;
//! extern crate holochain_agent;
//!
//! use holochain_core_api::*;
//! use holochain_dna::Dna;
//! use holochain_agent::Agent;
//! use std::sync::{Arc, Mutex};
//! use holochain_core::context::Context;
//! use holochain_core::logger::SimpleLogger;
//! use holochain_core::persister::SimplePersister;
//!
//! // instantiate a new app
//!
//! // need to get to something like this:
//! //let dna = holochain_dna::from_package_file("mydna.hcpkg");
//!
//! // but for now:
//! let dna = Dna::new();
//! let agent = Agent::from_string("bob".to_string());
//! let context = Context::new(
//!     agent,
//!     Arc::new(Mutex::new(SimpleLogger {})),
//!     Arc::new(Mutex::new(SimplePersister::new())),
//! );
//! let mut hc = Holochain::new(dna,Arc::new(context)).unwrap();
//!
//! // start up the app
//! hc.start().expect("couldn't start the app");
//!
//! // call a function in the app
//! hc.call("test_zome","test_cap","some_fn","{}");
//!
//! // get the state
//! {
//!     let state = hc.state();
//!
//!     // do some other stuff with the state here
//!     // ...
//! }
//!
//! // stop the app
//! hc.stop().expect("couldn't stop the app");
//!
//!```

extern crate holochain_core;
extern crate holochain_dna;
#[cfg(test)]
extern crate test_utils;

use holochain_core::{
    action::{Action, ActionWrapper},
    context::Context,
    error::HolochainError,
    instance::{Instance, RECV_DEFAULT_TIMEOUT_MS},
    nucleus::{call_and_wait_for_result, state::NucleusStatus, ZomeFnCall},
    state::State,
};
use holochain_dna::Dna;
use std::sync::{mpsc::channel, Arc};

/// contains a Holochain application instance
pub struct Holochain {
    instance: Instance,
    #[allow(dead_code)]
    context: Arc<Context>,
    active: bool,
}

impl Holochain {
    /// create a new Holochain instance
    pub fn new(dna: Dna, context: Arc<Context>) -> Result<Self, HolochainError> {
        let mut instance = Instance::new();
        let name = dna.name.clone();

        let action = ActionWrapper::new(Action::InitApplication(dna));
        instance.start_action_loop(context.clone());

        let (sender, receiver) = channel();

        instance.dispatch_with_observer(action, move |state: &State| {
            let nucleus_state = state.nucleus();
            if nucleus_state.has_initialized() || nucleus_state.has_initialization_failed() {
                sender
                    .send(nucleus_state.status())
                    .expect("test channel must be open");
                true
            } else {
                false
            }
        });

        // TODO: what is the right timeout?
        // had to increase this number when merging develop into feature branch 221-dna-improvements
        // https://github.com/holochain/holochain-rust/pull/253
        // solving ticket https://github.com/holochain/holochain-rust/issues/221
        match receiver.recv_timeout(RECV_DEFAULT_TIMEOUT_MS) {
            Ok(status) => match status {
                NucleusStatus::InitializationFailed(err) => Err(HolochainError::ErrorGeneric(err)),
                _ => {
                    context.log(&format!("{} instantiated", name))?;
                    let app = Holochain {
                        instance,
                        context,
                        active: false,
                    };
                    Ok(app)
                }
            },
            Err(err) => {
                // TODO: what kind of cleanup to do on an initialization timeout?
                // see #120:  https://waffle.io/holochain/org/cards/5b43704336bf54001bceeee0
                Err(HolochainError::ErrorGeneric(err.to_string()))
            }
        }
    }

    /// activate the Holochain instance
    pub fn start(&mut self) -> Result<(), HolochainError> {
        if self.active {
            return Err(HolochainError::InstanceActive);
        }
        self.active = true;
        Ok(())
    }

    /// deactivate the Holochain instance
    pub fn stop(&mut self) -> Result<(), HolochainError> {
        if !self.active {
            return Err(HolochainError::InstanceNotActive);
        }
        self.active = false;
        Ok(())
    }

    /// call a function in a zome
    pub fn call(
        &mut self,
        zome: &str,
        cap: &str,
        fn_name: &str,
        params: &str,
    ) -> Result<String, HolochainError> {
        if !self.active {
            return Err(HolochainError::InstanceNotActive);
        }

        let zome_call = ZomeFnCall::new(&zome, &cap, &fn_name, &params);

        call_and_wait_for_result(zome_call, &mut self.instance)
    }

    /// checks to see if an instance is active
    pub fn active(&self) -> bool {
        self.active
    }

    /// return
    pub fn state(&mut self) -> Result<State, HolochainError> {
        Ok(self.instance.state().clone())
    }
}

#[cfg(test)]
mod tests {
    extern crate holochain_agent;
    use super::*;
    use holochain_core::{
        context::Context,
        nucleus::ribosome::{callback::Callback, Defn},
        persister::SimplePersister,
    };
    use holochain_dna::Dna;
    use std::sync::{Arc, Mutex};
    use test_utils::{
        create_test_cap_with_fn_name, create_test_dna_with_cap, create_test_dna_with_wat,
        create_wasm_from_file,
    };

    // TODO: TestLogger duplicated in test_utils because:
    //  use holochain_core::{instance::tests::TestLogger};
    // doesn't work.
    // @see https://github.com/holochain/holochain-rust/issues/185
    fn test_context(agent_name: &str) -> (Arc<Context>, Arc<Mutex<test_utils::TestLogger>>) {
        let agent = holochain_agent::Agent::from_string(agent_name.to_string());
        let logger = test_utils::test_logger();
        (
            Arc::new(Context::new(
                agent,
                logger.clone(),
                Arc::new(Mutex::new(SimplePersister::new())),
            )),
            logger,
        )
    }

    #[test]
    fn can_instantiate() {
        let mut dna = Dna::new();
        dna.name = "TestApp".to_string();
        let (context, test_logger) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());

        match result {
            Ok(hc) => {
                assert_eq!(hc.instance.state().nucleus().dna(), Some(dna));
                assert!(!hc.active);
                assert_eq!(hc.context.agent.to_string(), "bob".to_string());
                assert!(hc.instance.state().nucleus().has_initialized());
                let test_logger = test_logger.lock().unwrap();
                assert_eq!(format!("{:?}", *test_logger), "[\"TestApp instantiated\"]");
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn fails_instantiate_if_genesis_fails() {
        let dna = create_test_dna_with_wat(
            "test_zome",
            Callback::Genesis.capability().as_str(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis") (param $p0 i32) (result i32)
                    i32.const 4
                )
                (data (i32.const 0)
                    "fail"
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let (context, _test_logger) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());

        match result {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err, HolochainError::ErrorGeneric("fail".to_string())),
        };
    }

    #[test]
    fn fails_instantiate_if_genesis_times_out() {
        let dna = create_test_dna_with_wat(
            "test_zome",
            Callback::Genesis.capability().as_str(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis") (param $p0 i32) (result i32)
                    (loop (br 0))
                    i32.const 0
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let (context, _test_logger) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());

        match result {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(
                err,
                HolochainError::ErrorGeneric("timed out waiting on channel".to_string())
            ),
        };
    }

    #[test]
    fn can_start_and_stop() {
        let dna = Dna::new();
        let (context, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();
        assert!(!hc.active());

        // stop when not active returns error
        let result = hc.stop();
        match result {
            Err(HolochainError::InstanceNotActive) => assert!(true),
            Ok(_) => assert!(false),
            Err(_) => assert!(false),
        }

        let result = hc.start();
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
        assert!(hc.active());

        // start when active returns error
        let result = hc.start();
        match result {
            Err(HolochainError::InstanceActive) => assert!(true),
            Ok(_) => assert!(false),
            Err(_) => assert!(false),
        }

        let result = hc.stop();
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
        assert!(!hc.active());
    }

    #[test]
    fn can_call() {
        let wat = r#"
(module
 (memory 1)
 (export "memory" (memory 0))
 (export "main" (func $func0))
 (func $func0 (param $p0 i32) (result i32)
       i32.const 16
       )
 (data (i32.const 0)
       "{\"holo\":\"world\"}"
       )
 )
"#;
        let dna = create_test_dna_with_wat("test_zome", "test_cap", Some(wat));
        let (context, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.call("test_zome", "test_cap", "main", "");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), HolochainError::InstanceNotActive);

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call("test_zome", "test_cap", "main", "");
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(result.ok().unwrap(), "{\"holo\":\"world\"}")
    }

    #[test]
    fn can_get_state() {
        let dna = Dna::new();
        let (context, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.state();
        match result {
            Ok(state) => {
                assert_eq!(state.nucleus().dna(), Some(dna));
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn can_call_test() {
        let wasm = create_wasm_from_file(
            "wasm-test/round_trip/target/wasm32-unknown-unknown/debug/round_trip.wasm",
        );
        let capability = create_test_cap_with_fn_name("test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call(
            "test_zome",
            "test_cap",
            "test",
            r#"{"input_int_val":2,"input_str_val":"fish"}"#,
        );
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            r#"{"input_int_val_plus2":4,"input_str_val_plus_dog":"fish.puppy"}"#
        );
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "wasm-test/commit/target/wasm32-unknown-unknown/debug/commit.wasm",
        );
        let capability = create_test_cap_with_fn_name("test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 3);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "test", r#"{}"#);
        println!("result = {:?}", result);

        // Expect fail because no validation function in wasm
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(result.ok().unwrap(), r#"{"hash":"fail"}"#);

        // Check in holochain instance's history that the commit event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 7);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit_err() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "wasm-test/commit/target/wasm32-unknown-unknown/debug/commit.wasm",
        );
        let capability = create_test_cap_with_fn_name("test_fail");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 3);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "test_fail", r#"{}"#);

        // Expect normal OK result with hash
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(result.ok().unwrap(), r#"{"hash":"fail"}"#);

        // Check in holochain instance's history that the commit event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "../core/src/nucleus/wasm-test/target/wasm32-unknown-unknown/debug/debug.wasm",
        );
        let capability = create_test_cap_with_fn_name("debug_hello");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, test_logger) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 3);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "debug_hello", r#"{}"#);
        assert!(result.unwrap().is_empty());

        let test_logger = test_logger.lock().unwrap();
        assert_eq!(
            format!("{:?}", *test_logger),
            "[\"TestApp instantiated\", \"\\\"Hello world!\\\"\"]"
        );
        // Check in holochain instance's history that the debug event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug_multiple() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "../core/src/nucleus/wasm-test/target/wasm32-unknown-unknown/debug/debug.wasm",
        );
        let capability = create_test_cap_with_fn_name("debug_multiple");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 3);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "debug_multiple", r#"{}"#);

        // TODO #165 - check runtime.print_output instead
        // Expect empty OK result
        assert!(result.unwrap().is_empty());

        // Check in holochain instance's history that the deb event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);
    }
}
