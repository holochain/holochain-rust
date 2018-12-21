//! `holochain_container_api` is a library for instantiating and using holochain instances that
//! each run a holochain DNA, DHT and source chain.
//!
//! The struct Holochain wraps everything needed to run such an instance.
//!
//! # Examples
//!
//! ``` rust
//! extern crate holochain_container_api;
//! extern crate holochain_core_types;
//! extern crate holochain_core;
//! extern crate holochain_net;
//! extern crate holochain_cas_implementations;
//! extern crate tempfile;
//! use holochain_container_api::{*, context_builder::ContextBuilder};
//! use holochain_core_types::{
//!     cas::content::Address,
//!     agent::AgentId,
//!     dna::{Dna, capabilities::CapabilityCall},
//!     json::JsonString};
//! use std::sync::Arc;
//! use tempfile::tempdir;
//!
//! // instantiate a new holochain instance
//!
//! // need to get to something like this:
//! //let dna = holochain_core_types::dna::from_package_file("mydna.hcpkg");
//!
//! // but for now:
//! let dna = Dna::new();
//! let dir = tempdir().unwrap();
//! let storage_directory_path = dir.path().to_str().unwrap();
//! let agent = AgentId::generate_fake("bob");
//! let context = ContextBuilder::new()
//!     .with_agent(agent)
//!     .with_file_storage(storage_directory_path)
//!     .expect("Tempdir should be accessible")
//!     .spawn();
//! let mut hc = Holochain::new(dna,Arc::new(context)).unwrap();
//!
//! // start up the holochain instance
//! hc.start().expect("couldn't start the holochain instance");
//!
//! // call a function in the zome code
//! hc.call("test_zome", Some(CapabilityCall::new("foo".to_string(), Address::from(""), None)), "some_fn", "{}");
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

use crate::error::{HolochainInstanceError, HolochainResult};
use futures::executor::block_on;
use holochain_core::{
    context::Context,
    instance::Instance,
    nucleus::{call_and_wait_for_result, ZomeFnCall},
    persister::{Persister, SimplePersister},
    state::State,
    workflows::application,
};
use holochain_core_types::{
    dna::{capabilities::CapabilityCall, Dna},
    error::HolochainError,
    json::JsonString,
};
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
    pub fn new(dna: Dna, context: Arc<Context>) -> HolochainResult<Self> {
        let instance = Instance::new(context.clone());
        Self::from_dna_and_context_and_instance(dna, context, instance)
    }

    fn from_dna_and_context_and_instance(
        dna: Dna,
        context: Arc<Context>,
        mut instance: Instance,
    ) -> HolochainResult<Self> {
        let name = dna.name.clone();
        instance.start_action_loop(context.clone());
        let result = block_on(application::initialize(
            &instance,
            Some(dna),
            context.clone(),
        ));
        match result {
            Ok(new_context) => {
                context.log(format!("{} instantiated", name));
                let hc = Holochain {
                    instance,
                    context: new_context.clone(),
                    active: false,
                };
                Ok(hc)
            }
            Err(err) => Err(HolochainInstanceError::InternalFailure(err)),
        }
    }

    pub fn load(_path: String, context: Arc<Context>) -> Result<Self, HolochainError> {
        let persister = SimplePersister::new(context.dht_storage.clone());
        let loaded_state = persister
            .load(context.clone())?
            .unwrap_or(State::new(context.clone()));
        let mut instance = Instance::from_state(loaded_state.clone());
        instance.start_action_loop(context.clone());
        let new_context = block_on(application::initialize(&instance, None, context.clone()))?;
        Ok(Holochain {
            instance,
            context: new_context.clone(),
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
        cap: Option<CapabilityCall>,
        fn_name: &str,
        params: &str,
    ) -> HolochainResult<JsonString> {
        if !self.active {
            return Err(HolochainInstanceError::InstanceNotActiveYet);
        }
        let zome_call = ZomeFnCall::new(&zome, cap, &fn_name, String::from(params));
        Ok(call_and_wait_for_result(zome_call, &mut self.instance)?)
    }

    /// checks to see if an instance is active
    pub fn active(&self) -> bool {
        self.active
    }

    /// return
    pub fn state(&self) -> Result<State, HolochainInstanceError> {
        Ok(self.instance.state().clone())
    }
}

#[cfg(test)]
mod tests {
    extern crate holochain_cas_implementations;

    use super::*;
    use context_builder::ContextBuilder;
    use holochain_core::{
        action::Action,
        context::Context,
        nucleus::ribosome::{callback::Callback, Defn},
        signal::{signal_channel, Signal, SignalReceiver},
    };
    use holochain_core_types::{
        agent::AgentId,
        cas::content::{Address, AddressableContent},
        dna::Dna,
    };
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    use test_utils::{
        create_test_cap_with_fn_name, create_test_dna_with_cap, create_test_dna_with_wat,
        create_wasm_from_file, expect_action, hc_setup_and_call_zome_fn,
    };

    // TODO: TestLogger duplicated in test_utils because:
    //  use holochain_core::{instance::tests::TestLogger};
    // doesn't work.
    // @see https://github.com/holochain/holochain-rust/issues/185
    fn test_context(
        agent_name: &str,
    ) -> (
        Arc<Context>,
        Arc<Mutex<test_utils::TestLogger>>,
        SignalReceiver,
    ) {
        let agent = AgentId::generate_fake(agent_name);
        let (signal_tx, signal_rx) = signal_channel();
        let logger = test_utils::test_logger();
        (
            Arc::new(
                ContextBuilder::new()
                    .with_agent(agent)
                    .with_logger(logger.clone())
                    .with_signals(signal_tx)
                    .with_file_storage(tempdir().unwrap().path().to_str().unwrap())
                    .unwrap()
                    .spawn(),
            ),
            logger,
            signal_rx,
        )
    }

    use std::{fs::File, io::prelude::*, path::MAIN_SEPARATOR};

    fn example_api_wasm_path() -> String {
        "wasm-test/target/wasm32-unknown-unknown/release/example_api_wasm.wasm".into()
    }

    fn example_api_wasm() -> Vec<u8> {
        create_wasm_from_file(&example_api_wasm_path())
    }

    fn example_capability_call() -> Option<CapabilityCall> {
        Some(CapabilityCall::new(
            "test_cap".to_string(),
            Address::from("test_token"),
            None,
        ))
    }

    #[test]
    fn can_instantiate() {
        let mut dna = Dna::new();
        dna.name = "TestApp".to_string();
        let (context, test_logger, _) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());
        assert!(result.is_ok());
        let hc = result.unwrap();
        assert_eq!(hc.instance.state().nucleus().dna(), Some(dna));
        assert!(!hc.active);
        assert_eq!(hc.context.agent_id.nick, "bob".to_string());
        let network_state = hc.context.state().unwrap().network().clone();
        assert_eq!(network_state.agent_id.is_some(), true);
        assert_eq!(network_state.dna_hash.is_some(), true);
        assert!(hc.instance.state().nucleus().has_initialized());
        let test_logger = test_logger.lock().unwrap();
        assert_eq!(format!("{:?}", *test_logger), "[\"TestApp instantiated\"]");
    }

    fn write_agent_state_to_file() -> String {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().to_str().unwrap();
        let tempfile = vec![path, "Agentstate.txt"].join(&*MAIN_SEPARATOR.to_string());
        let mut file = File::create(tempfile).unwrap();
        file.write_all(b"{\"top_chain_header\":{\"entry_type\":\"AgentId\",\"entry_address\":\"Qma6RfzvZRL127UCEVEktPhQ7YSS1inxEFw7SjEsfMJcrq\",\"sources\":[\"sandwich--------------------------------------------------------------------------AAAEqzh28L\"],\"entry_signatures\":[\"fake-signature\"],\"link\":null,\"link_same_type\":null,\"timestamp\":\"2018-10-11T03:23:38+00:00\"}}").unwrap();
        path.to_string()
    }
    #[test]
    fn can_load() {
        let path = write_agent_state_to_file();
        let (context, _, _) = test_context("bob");
        let result = Holochain::load(path, context.clone());
        assert!(result.is_ok());
        let loaded_holo = result.unwrap();
        assert!(!loaded_holo.active);
        assert_eq!(loaded_holo.context.agent_id.nick, "bob".to_string());
        let network_state = loaded_holo.context.state().unwrap().network().clone();
        assert_eq!(network_state.agent_id.is_some(), true);
        assert_eq!(network_state.dna_hash.is_some(), true);
        assert!(loaded_holo.instance.state().nucleus().has_initialized());
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
                    i32.const 9
                )
                (data (i32.const 0)
                    "fail"
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let (context, _test_logger, _) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());
        assert!(result.is_err());
        assert_eq!(
            HolochainInstanceError::from(HolochainError::ErrorGeneric("\"Genesis\"".to_string())),
            result.err().unwrap(),
        );
    }

    #[test]
    fn fails_instantiate_if_genesis_times_out() {
        // let dna = create_test_dna_with_wat(
        //     "test_zome",
        //     Callback::Genesis.capability().as_str(),
        //     Some(
        //         r#"
        //     (module
        //         (memory (;0;) 17)
        //         (func (export "genesis") (param $p0 i32) (result i32)
        //             (loop (br 0))
        //             i32.const 0
        //         )
        //         (export "memory" (memory 0))
        //     )
        // "#,
        //     ),
        // );
        //
        // let (context, _test_logger, _) = test_context("bob");
        // let result = Holochain::new(dna.clone(), context.clone());
        // assert!(result.is_err());
        // assert_eq!(
        //     HolochainInstanceError::from(HolochainError::ErrorGeneric(
        //         "Timeout while initializing".to_string()
        //     )),
        //     result.err().unwrap(),
        // );
    }

    #[test]
    fn can_start_and_stop() {
        let dna = Dna::new();
        let (context, _, _) = test_context("bob");
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
        let (context, _, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.call("test_zome", example_capability_call(), "main", "");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            HolochainInstanceError::InstanceNotActiveYet
        );

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call("test_zome", example_capability_call(), "main", "");
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from("{\"holo\":\"world\"}")
        );
    }

    #[test]
    fn can_get_state() {
        let dna = Dna::new();
        let (context, _, _) = test_context("bob");
        let hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.state();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nucleus().dna(), Some(dna));
    }

    #[test]
    fn can_call_test() {
        let wasm = example_api_wasm();
        let capability = create_test_cap_with_fn_name("round_trip_test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call(
            "test_zome",
            example_capability_call(),
            "round_trip_test",
            r#"{"input_int_val":2,"input_str_val":"fish"}"#,
        );
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from(r#"{"input_int_val_plus2":4,"input_str_val_plus_dog":"fish.puppy"}"#),
        );
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let capability = create_test_cap_with_fn_name("commit_test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _, signal_rx) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");

        expect_action(&signal_rx, |action| {
            if let Action::InitNetwork(_) = action {
                true
            } else {
                false
            }
        })
        .unwrap();

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call(
            "test_zome",
            example_capability_call(),
            "commit_test",
            r#"{}"#,
        );

        // Expect fail because no validation function in wasm
        assert!(result.is_ok(), "result = {:?}", result);
        // @TODO fragile test!
        assert_ne!(
            result.clone().ok().unwrap(),
            JsonString::from("{\"Err\":\"Argument deserialization failed\"}")
        );

        expect_action(&signal_rx, |action| {
            if let Action::Commit(_) = action {
                true
            } else {
                false
            }
        })
        .unwrap();
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit_err() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let capability = create_test_cap_with_fn_name("commit_fail_test");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);
        let (context, _, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call(
            "test_zome",
            example_capability_call(),
            "commit_fail_test",
            r#"{}"#,
        );
        println!("can_call_commit_err result: {:?}", result);

        // Expect normal OK result with hash
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from("{\"Err\":\"Argument deserialization failed\"}"),
        );

        // Check in holochain instance's history that the commit event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 7);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let capability = create_test_cap_with_fn_name("debug_hello");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, test_logger, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");

        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call(
            "test_zome",
            example_capability_call(),
            "debug_hello",
            r#"{}"#,
        );

        assert_eq!(Ok(JsonString::null()), result,);
        let test_logger = test_logger.lock().unwrap();
        assert_eq!(
            "[\"TestApp instantiated\", \"zome_log:DEBUG: \\\'\\\"Hello world!\\\"\\\'\", \"Zome Function \\\'debug_hello\\\' returned: Success\"]",
            format!("{:?}", test_logger.log),
        );
        // Check in holochain instance's history that the debug event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 7);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug_multiple() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let capability = create_test_cap_with_fn_name("debug_multiple");
        let dna = create_test_dna_with_cap("test_zome", "test_cap", &capability, &wasm);

        let (context, test_logger, _) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 5);

        // Call the exposed wasm function that calls the Commit API function
        let result = hc.call(
            "test_zome",
            example_capability_call(),
            "debug_multiple",
            r#"{}"#,
        );

        // Expect Success as result
        println!("result = {:?}", result);
        assert_eq!(Ok(JsonString::null()), result,);

        let test_logger = test_logger.lock().unwrap();

        assert_eq!(
            "[\"TestApp instantiated\", \"zome_log:DEBUG: \\\'\\\"Hello\\\"\\\'\", \"zome_log:DEBUG: \\\'\\\"world\\\"\\\'\", \"zome_log:DEBUG: \\\'\\\"!\\\"\\\'\", \"Zome Function \\\'debug_multiple\\\' returned: Success\"]",
            format!("{:?}", test_logger.log),
        );

        // Check in holochain instance's history that the deb event has been processed
        // @TODO don't use history length in tests
        // @see https://github.com/holochain/holochain-rust/issues/195
        assert_eq!(hc.state().unwrap().history.len(), 7);
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn call_debug_stacked() {
        let call_result =
            hc_setup_and_call_zome_fn(&example_api_wasm_path(), "debug_stacked_hello");
        assert_eq!(
            JsonString::from("{\"value\":\"fish\"}"),
            call_result.unwrap()
        );
    }

    #[test]
    fn can_receive_action_signals() {
        use holochain_core::action::Action;
        use std::time::Duration;
        let wasm = include_bytes!(
            "../wasm-test/target/wasm32-unknown-unknown/release/example_api_wasm.wasm"
        );
        let capability = test_utils::create_test_cap_with_fn_name("commit_test");
        let mut dna =
            test_utils::create_test_dna_with_cap("test_zome", "test_cap", &capability, wasm);
        dna.uuid = "can_receive_action_signals".into();
        let (context, _, signal_rx) = test_context("alex");
        let timeout = 1000;
        let mut hc = Holochain::new(dna.clone(), context).unwrap();
        hc.start().expect("couldn't start");
        hc.call(
            "test_zome",
            example_capability_call(),
            "commit_test",
            r#"{}"#,
        )
        .unwrap();

        'outer: loop {
            let msg_publish = signal_rx
                .recv_timeout(Duration::from_millis(timeout))
                .expect("no more signals to receive (outer)");
            if let Signal::Internal(Action::Publish(address)) = msg_publish {
                loop {
                    let msg_hold = signal_rx
                        .recv_timeout(Duration::from_millis(timeout))
                        .expect("no more signals to receive (inner)");
                    if let Signal::Internal(Action::Hold(entry)) = msg_hold {
                        assert_eq!(address, entry.address());
                        break 'outer;
                    }
                }
            }
        }
    }
}
