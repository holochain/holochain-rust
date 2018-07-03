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
//! let agent = Agent::from_string("bob");
//! let context = Context {
//!     agent: agent,
//!     logger: Arc::new(Mutex::new(SimpleLogger {})),
//!     persister: Arc::new(Mutex::new(SimplePersister::new())),
//! };
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

extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_dna;

use holochain_core::{
    context::Context, error::HolochainError, instance::Instance,
    nucleus::{call_and_wait_for_result, Action::*, FunctionCall}, state::{Action::*, State},
};
use holochain_dna::Dna;
use std::sync::Arc;

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
        let action = Nucleus(InitApplication(dna));
        instance.start_action_loop();
        instance.dispatch_and_wait(action);
        context.log(&format!("{} instantiated", name))?;
        let app = Holochain {
            instance,
            context,
            active: false,
        };
        Ok(app)
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
    pub fn call<T: Into<String>>(
        &mut self,
        zome: T,
        cap: T,
        fn_name: T,
        params: T,
    ) -> Result<String, HolochainError> {
        if !self.active {
            return Err(HolochainError::InstanceNotActive);
        }

        let call = FunctionCall::new(zome.into(), cap.into(), fn_name.into(), params.into());

        call_and_wait_for_result(call, &mut self.instance)
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
    use super::*;
    use holochain_agent::Agent as HCAgent;
    use holochain_core::{
        context::Context, logger::Logger, persister::SimplePersister,
        test_utils::{create_test_dna_with_wasm, create_test_dna_with_wat, create_wasm_from_file},
    };
    use std::{
        fmt, sync::{Arc, Mutex},
    };

    #[derive(Clone)]
    struct TestLogger {
        log: Vec<String>,
    }

    impl Logger for TestLogger {
        fn log(&mut self, msg: String) {
            self.log.push(msg);
        }
    }

    // trying to get a way to print out what has been logged for tests without a read function.
    // this currently fails
    impl fmt::Debug for TestLogger {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self.log[0])
        }
    }

    fn test_context(agent: holochain_agent::Agent) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
        let logger = Arc::new(Mutex::new(TestLogger { log: Vec::new() }));
        (
            Arc::new(Context {
                agent: agent,
                logger: logger.clone(),
                persister: Arc::new(Mutex::new(SimplePersister::new())),
            }),
            logger,
        )
    }

    #[test]
    fn can_instantiate() {
        let mut dna = Dna::new();
        dna.name = "TestApp".to_string();
        let agent = HCAgent::from_string("bob");
        let (context, test_logger) = test_context(agent.clone());
        let result = Holochain::new(dna.clone(), context.clone());

        match result {
            Ok(hc) => {
                assert_eq!(hc.instance.state().nucleus().dna(), Some(dna));
                assert!(!hc.active);
                assert_eq!(hc.context.agent, agent);
                // TODO #61 - Should instantiation also initialize?
                // assert!(hc.instance.state().nucleus().has_initialized());
                let test_logger = test_logger.lock().unwrap();
                assert_eq!(format!("{:?}", *test_logger), "\"TestApp instantiated\"");
            }
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn can_start_and_stop() {
        let dna = Dna::new();
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
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
 (export "hello_dispatch" (func $func0))
 (func $func0 (param $p0 i32) (param $p1 i32) (result i32)
       i32.const 16
       )
 (data (i32.const 0)
       "{\"holo\":\"world\"}"
       )
 )
"#;
        let dna = create_test_dna_with_wat("test_zome".to_string(), "test_cap".to_string(),Some(wat));
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.call("test_zome", "test_cap", "hello", "");
        match result {
            Err(HolochainError::InstanceNotActive) => assert!(true),
            Err(_) => assert!(false),
            Ok(_) => assert!(false),
        }

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call("test_zome", "test_cap", "hello", "");
        println!("{:#?}", result);
        match result {
            Ok(result) => assert_eq!(result, "{\"holo\":\"world\"}"),
            Err(_) => assert!(false),
        };
    }

    #[test]
    fn can_get_state() {
        let dna = Dna::new();
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
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
        let dna = create_test_dna_with_wasm("test_zome".to_string(), "test_cap".to_string(),wasm);
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call(
            "test_zome",
            "test_cap",
            "test",
            r#"{"input_int_val":2,"input_str_val":"fish"}"#,
        );
        match result {
            Ok(result) => assert_eq!(
                result,
                r#"{"input_int_val_plus2":4,"input_str_val_plus_dog":"fish.puppy"}"#
            ),
            Err(_) => assert!(false),
        };
    }

}
