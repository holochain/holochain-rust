//! `holochain_core_api` is a library for instantiating and using a holochain instance that
//! runs a holochain DNA, DHT and source chain.
//!
//! # Examples
//!
//! ``` rust
//! extern crate holochain_core;
//! extern crate holochain_core_api;
//! extern crate holochain_dna;
//! extern crate holochain_agent;
//! extern crate holochain_cas_implementations;
//! extern crate tempfile;
//! use holochain_core_api::*;
//! use holochain_dna::Dna;
//! use holochain_agent::Agent;
//! use std::sync::{Arc, Mutex};
//! use holochain_core::context::Context;
//! use holochain_core::logger::SimpleLogger;
//! use holochain_core::persister::SimplePersister;
//! use self::holochain_cas_implementations::{
//!        cas::file::FilesystemStorage, eav::file::EavFileStorage,
//! };
//! use tempfile::tempdir;
//!
//! // instantiate a new holochain instance
//!
//! // need to get to something like this:
//! //let dna = holochain_dna::from_package_file("mydna.hcpkg");
//!
//! // but for now:
//! let dna = Dna::new();
//! let agent = Agent::from("bob".to_string());
//! let context = Context::new(
//!     agent,
//!     Arc::new(Mutex::new(SimpleLogger {})),
//!     Arc::new(Mutex::new(SimplePersister::new(String::from("Agent Name")))),
//!     FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
//!     EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string()).unwrap(),
//!  ).unwrap();
//! let mut hc = Holochain::new(dna,Arc::new(context)).unwrap();
//!
//! // start up the holochain instance
//! hc.start().expect("couldn't start the holochain instance");
//!
//! // call a function in the zome code
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
//! // stop the holochain instance
//! hc.stop().expect("couldn't stop the holochain instance");
//!
//!```

extern crate futures;
extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_dna;
extern crate tempfile;
#[cfg(test)]
extern crate test_utils;

pub mod error;

use error::{HolochainInstanceError, HolochainResult};
use futures::executor::block_on;
use holochain_core::{
    context::Context,
    instance::Instance,
    nucleus::{actions::initialize::initialize_application, call_and_wait_for_result, ZomeFnCall},
    persister::{Persister, SimplePersister},
    state::State,
};
use holochain_core_types::error::HolochainError;
use holochain_dna::Dna;
use std::sync::{Arc, RwLock};

/// contains a Holochain application instance
pub struct Holochain {
    instance: Instance,
    #[allow(dead_code)]
    context: Arc<Context>,
    active: bool,
}

impl Holochain {
    /// create a new Holochain instance
    pub fn new(dna: Dna, context: Arc<Context>) -> HolochainResult<Self> {
        let mut instance = Instance::new(context.clone());
        let name = dna.name.clone();
        instance.start_action_loop(context.clone());
        let context = instance.initialize_context(context);
        match block_on(initialize_application(dna, context.clone())) {
            Ok(_) => {
                context.log(&format!("{} instantiated", name))?;
                let hc = Holochain {
                    instance,
                    context,
                    active: false,
                };
                Ok(hc)
            }
            Err(err_str) => Err(HolochainInstanceError::InternalFailure(
                HolochainError::ErrorGeneric(err_str),
            )),
        }
    }

    pub fn load(path: String, context: Arc<Context>) -> Result<Self, HolochainError> {
        let mut new_context = (*context).clone();
        let persister = SimplePersister::new(format!("{}/state", path));
        let loaded_state = persister
            .load(context.clone())
            .unwrap_or(Some(State::new(context.clone())))
            .unwrap();
        let mut instance = Instance::from_state(loaded_state);
        instance.start_action_loop(context.clone());
        Ok(Holochain {
            instance,
            context: context.clone(),
            active: false,
        })
    }

    /// activate the Holochain instance
    pub fn start(&mut self) -> Result<(), HolochainInstanceError> {
        if self.active {
            return Err(HolochainInstanceError::InstanceAlreadyActive);
        }
        self.active = true;
        Ok(())
    }

    /// deactivate the Holochain instance
    pub fn stop(&mut self) -> Result<(), HolochainInstanceError> {
        if !self.active {
            return Err(HolochainInstanceError::InstanceNotActiveYet);
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
    ) -> HolochainResult<String> {
        if !self.active {
            return Err(HolochainInstanceError::InstanceNotActiveYet);
        }
        let zome_call = ZomeFnCall::new(&zome, &cap, &fn_name, &params);
        Ok(call_and_wait_for_result(zome_call, &mut self.instance)?)
    }

    /// checks to see if an instance is active
    pub fn active(&self) -> bool {
        self.active
    }

    /// return
    pub fn state(&mut self) -> Result<State, HolochainInstanceError> {
        Ok(self.instance.state().clone())
    }
}

#[cfg(test)]
mod tests {
    extern crate holochain_cas_implementations;

    use self::holochain_cas_implementations::{
        cas::file::FilesystemStorage, eav::file::EavFileStorage,
    };
    use super::*;
    extern crate holochain_agent;
    use holochain_core::{
        context::Context,
        nucleus::ribosome::{callback::Callback, Defn},
        persister::SimplePersister,
    };
    use holochain_dna::Dna;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    use test_utils::{
        create_test_cap_with_fn_name, create_test_dna_with_cap, create_test_dna_with_wat,
        create_wasm_from_file, hc_setup_and_call_zome_fn,
    };

    // TODO: TestLogger duplicated in test_utils because:
    //  use holochain_core::{instance::tests::TestLogger};
    // doesn't work.
    // @see https://github.com/holochain/holochain-rust/issues/185
    fn test_context(agent_name: &str) -> (Arc<Context>, Arc<Mutex<test_utils::TestLogger>>) {
        let agent = holochain_agent::Agent::from(agent_name.to_string());
        let logger = test_utils::test_logger();
        (
            Arc::new(
                Context::new(
                    agent,
                    logger.clone(),
                    Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
                    FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
                    EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                        .unwrap(),
                ).unwrap(),
            ),
            logger,
        )
    }

    #[test]
    fn can_instantiate() {
        let mut dna = Dna::new();
        dna.name = "TestApp".to_string();
        let (context, test_logger) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());

        assert!(result.is_ok());
        let hc = result.unwrap();

        assert_eq!(hc.instance.state().nucleus().dna(), Some(dna));
        assert!(!hc.active);
        assert_eq!(hc.context.agent.to_string(), "bob".to_string());
        assert!(hc.instance.state().nucleus().has_initialized());
        let test_logger = test_logger.lock().unwrap();
        assert_eq!(format!("{:?}", *test_logger), "[\"TestApp instantiated\"]");
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
        assert!(result.is_err());
        assert_eq!(
            HolochainInstanceError::from(HolochainError::ErrorGeneric("fail".to_string())),
            result.err().unwrap(),
        );
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
        assert!(result.is_err());
        assert_eq!(
            HolochainInstanceError::from(HolochainError::ErrorGeneric(
                "Timeout while initializing".to_string()
            )),
            result.err().unwrap(),
        );
    }

    #[test]
    fn can_start_and_stop() {
        let dna = Dna::new();
        let (context, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();
        assert!(!hc.active());

        // stop when not active returns error
        let result = hc.stop();
        assert_eq!(
            HolochainInstanceError::InstanceNotActiveYet,
            result.err().unwrap()
        );

        let result = hc.start();
        assert!(result.is_ok());
        assert!(hc.active());

        // start when active returns error
        let result = hc.start();
        assert!(result.is_err());
        assert_eq!(
            HolochainInstanceError::InstanceAlreadyActive,
            result.err().unwrap()
        );

        let result = hc.stop();
        assert!(result.is_ok());
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
        assert_eq!(
            result.err().unwrap(),
            HolochainInstanceError::InstanceNotActiveYet
        );

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
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nucleus().dna(), Some(dna));
    }

    #[test]
    fn can_call_test() {
        let wasm = create_wasm_from_file(
            "wasm-test/round_trip/target/wasm32-unknown-unknown/release/round_trip.wasm",
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
            "wasm-test/commit/target/wasm32-unknown-unknown/release/commit.wasm",
        );
        let capability = create_test_cap_with_fn_name("test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 4);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "test", r#"{}"#);

        // Expect fail because no validation function in wasm
        assert!(result.is_ok(), "result = {:?}", result);
        assert_ne!(
            result.clone().ok().unwrap(),
            "{\"Err\":\"Argument deserialization failed\"}"
        );

        // Check in holochain instance's history that the commit event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 8);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit_err() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "wasm-test/commit/target/wasm32-unknown-unknown/release/commit.wasm",
        );
        let capability = create_test_cap_with_fn_name("test_fail");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 4);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "test_fail", r#"{}"#);

        // Expect normal OK result with hash
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            "{\"Err\":\"Argument deserialization failed\"}"
        );

        // Check in holochain instance's history that the commit event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 6);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "../core/src/nucleus/wasm-test/target/wasm32-unknown-unknown/release/debug.wasm",
        );
        let capability = create_test_cap_with_fn_name("debug_hello");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, test_logger) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 4);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "debug_hello", r#"{}"#);
        assert!(result.unwrap().is_empty());

        let test_logger = test_logger.lock().unwrap();
        assert_eq!(
            "[\"TestApp instantiated\", \"zome_log:DEBUG: \\\'\\\"Hello world!\\\"\\\'\", \"Zome Function \\\'debug_hello\\\' returned: Success\"]",
            format!("{:?}", test_logger.log),
        );
        // Check in holochain instance's history that the debug event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 6);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug_multiple() {
        // Setup the holochain instance
        let wasm = create_wasm_from_file(
            "../core/src/nucleus/wasm-test/target/wasm32-unknown-unknown/release/debug.wasm",
        );
        let capability = create_test_cap_with_fn_name("debug_multiple");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, test_logger) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 4);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call("test_zome", "test_cap", "debug_multiple", r#"{}"#);

        // Expect a string as result
        assert!(result.unwrap().is_empty());

        let test_logger = test_logger.lock().unwrap();
        assert_eq!(
            "[\"TestApp instantiated\", \"zome_log:DEBUG: \\\'\\\"Hello\\\"\\\'\", \"zome_log:DEBUG: \\\'\\\"world\\\"\\\'\", \"zome_log:DEBUG: \\\'\\\"!\\\"\\\'\", \"Zome Function \\\'debug_multiple\\\' returned: Success\"]",
            format!("{:?}", test_logger.log),
        );

        // Check in holochain instance's history that the deb event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 6);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn call_debug_stacked() {
        let call_result = hc_setup_and_call_zome_fn(
            "../core/src/nucleus/wasm-test/target/wasm32-unknown-unknown/release/debug.wasm",
            "debug_stacked_hello",
        );
        assert_eq!("{\"value\":\"fish\"}", call_result.unwrap());
    }
}
