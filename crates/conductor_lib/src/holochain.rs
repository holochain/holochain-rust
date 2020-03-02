//! `holochain_conductor_lib` is a library for instantiating and using holochain instances that
//! each run a holochain DNA, DHT and source chain.
//!
//! The struct Holochain wraps everything needed to run such an instance.
//!
//! # Examples
//!
//! ``` rust
//! extern crate holochain_conductor_lib;
//! extern crate holochain_core_types;
//! extern crate holochain_core;
//! extern crate holochain_locksmith;
//! extern crate holochain_net;
//! extern crate holochain_json_api;
//! extern crate holochain_persistence_api;
//! extern crate holochain_persistence_mem;
//! extern crate holochain_dpki;
//! extern crate lib3h_sodium;
//! extern crate tempfile;
//! extern crate test_utils;
//! use holochain_conductor_lib::{*, context_builder::ContextBuilder};
//! use holochain_core_types::{
//!     agent::AgentId,
//!     dna::{Dna, capabilities::CapabilityRequest,},
//!     signature::Signature
//! };
//! use holochain_locksmith::Mutex;
//! use holochain_persistence_api::{
//!     cas::content::Address,
//! };
//! use holochain_json_api::json::JsonString;
//! use holochain_dpki::{key_bundle::KeyBundle, seed::SeedType, SEED_SIZE};
//! use lib3h_sodium::secbuf::SecBuf;
//! use test_utils;
//!
//! use std::sync::Arc;
//! use tempfile::tempdir;
//!
//! // Instantiate a new holochain instance
//!
//! // Need to get to something like this:
//! // let dna = holochain_core_types::dna::from_package_file("mydna.dna.json");
//!
//! // But for now:
//! let dna = test_utils::create_arbitrary_test_dna();
//! let dir = tempdir().unwrap();
//! let storage_directory_path = dir.path().to_str().unwrap();
//!
//! // We need to provide a cryptographic key that represents the agent.
//! // Creating a new random one on the fly:
//! let mut seed = SecBuf::with_insecure(SEED_SIZE);
//! seed.randomize();
//!
//! let keybundle = KeyBundle::new_from_seed_buf(&mut seed).unwrap();
//!
//! // The keybundle's public part is the agent's address
//! let agent = AgentId::new("bob", keybundle.get_id());
//!
//! // The instance needs a conductor API with at least the signing callback:
//! let conductor_api = interface::ConductorApiBuilder::new()
//!     .with_agent_signature_callback(Arc::new(Mutex::new(keybundle)))
//!     .spawn();
//!
//! // The conductor API, together with the storage and the agent ID
//! // constitute the instance's context:
//! let context = ContextBuilder::new()
//!     .with_agent(agent)
//!     .with_conductor_api(conductor_api)
//!     .with_file_storage(storage_directory_path)
//!     .expect("Tempdir should be accessible")
//!     .spawn();
//!
//! let mut hc = Holochain::new(dna,Arc::new(context)).unwrap();
//!
//! // start up the holochain instance
//! hc.start().expect("couldn't start the holochain instance");
//!
//! // call a function in the zome code
//! hc.call("test_zome", CapabilityRequest::new(Address::from("some_token"), Address::from("caller"), Signature::fake()), "some_fn", "{}");
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

use crate::{
    error::{HolochainInstanceError, HolochainResult},
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core::{
    context::Context,
    instance::Instance,
    nucleus::{call_zome_function, ZomeFnCall},
    persister::{Persister, SimplePersister},
    wasm_engine::{WasmCallData},
};
use holochain_core_types::{
    dna::{capabilities::CapabilityRequest, Dna},
    error::HolochainError,
};
use holochain_wasm_types::wasm_string::WasmString;
use holochain_json_api::json::JsonString;

use holochain_core::{
    state::StateWrapper,
    state_dump::{address_to_content_and_type, StateDump},
};
use holochain_persistence_api::cas::content::Address;
use jsonrpc_core::IoHandler;
use std::sync::Arc;
use holochain_metrics::with_latency_publishing;

/// contains a Holochain application instance
pub struct Holochain {
    instance: Option<Instance>,
    context: Option<Arc<Context>>,
    active: bool,
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CONDUCTOR_LIB)]
impl Holochain {
    /// create a new Holochain instance.  Ensure that they are built w/ the same
    /// HDK Version, or log a warning.
    pub fn new(dna: Dna, context: Arc<Context>) -> HolochainResult<Self> {
        let instance = Instance::new(context.clone());

        for zome in dna.zomes.values() {
            let call_data = WasmCallData::DirectCall("__hdk_hdk_version".to_string(), zome.code.code.clone());

            let hdk_version: WasmString = match holochain_core::wasm_engine::guest::call(
                &mut call_data.instance()?,
                &call_data.fn_name(),
                (),
            ) {
                Ok(v) => v,
                Err(e) => return Err(HolochainInstanceError::InternalFailure(HolochainError::Wasm(e))),
            };

            if hdk_version.to_string()
                != holochain_core_types::hdk_version::HDK_VERSION.to_string()
            {
                eprintln!("WARNING! The HDK Version of the runtime and the zome don't match.");
                eprintln!(
                    "Runtime HDK Version: {}",
                    holochain_core_types::hdk_version::HDK_VERSION.to_string()
                );
                eprintln!("Zome HDK Version: {}", hdk_version.to_string());
            }
        }

        Self::from_dna_and_context_and_instance(dna, context, instance)
    }

    fn from_dna_and_context_and_instance(
        dna: Dna,
        context: Arc<Context>,
        mut instance: Instance,
    ) -> HolochainResult<Self> {
        let name = dna.name.clone();
        let result = instance.initialize(Some(dna), context.clone());

        match result {
            Ok(new_context) => {
                log_debug!(context, "conductor: {} instantiated", name);
                let hc = Holochain {
                    instance: Some(instance),
                    context: Some(new_context),
                    active: false,
                };
                Ok(hc)
            }
            Err(err) => Err(HolochainInstanceError::InternalFailure(err)),
        }
    }

    pub fn load(context: Arc<Context>) -> Result<Self, HolochainError> {
        let persister = SimplePersister::new(context.dht_storage.clone());
        let loaded_state = persister.load(context.clone())?.ok_or_else(|| {
            HolochainError::ErrorGeneric("State could not be loaded due to NoneError".to_string())
        })?;
        let mut instance = Instance::from_state(loaded_state, context.clone());
        let new_context = instance.initialize(None, context)?;
        Ok(Holochain {
            instance: Some(instance),
            context: Some(new_context),
            active: false,
        })
    }

    pub fn check_instance(&self) -> Result<(), HolochainInstanceError> {
        if self.instance.is_none() || self.context.is_none() {
            Err(HolochainInstanceError::InstanceNotInitialized)
        } else {
            Ok(())
        }
    }

    pub fn check_active(&self) -> Result<(), HolochainInstanceError> {
        if !self.active {
            Err(HolochainInstanceError::InstanceNotActiveYet)
        } else {
            Ok(())
        }
    }

    pub fn kill(&mut self) {
        let _ = self.stop();
        self.instance = None;
        self.context = None;
    }

    /// activate the Holochain instance
    pub fn start(&mut self) -> Result<(), HolochainInstanceError> {
        self.check_instance()?;
        if self.active {
            Err(HolochainInstanceError::InstanceAlreadyActive)
        } else {
            self.active = true;
            Ok(())
        }
    }

    /// deactivate the Holochain instance
    pub fn stop(&mut self) -> Result<(), HolochainInstanceError> {
        self.check_instance()?;
        self.check_active()?;

        let context = self.context.as_ref().unwrap();
        if let Err(err) = context.block_on(self.instance.as_ref().unwrap().shutdown_network()) {
            log_error!(context, "Error shutting down network: {:?}", err);
        }
        self.instance.as_ref().unwrap().stop_action_loop();
        self.active = false;
        Ok(())
    }

    fn call_inner(
        context: Arc<Context>,
        zome: &str,
        cap: CapabilityRequest,
        fn_name: &str,
        params: &str,
    ) -> HolochainResult<JsonString> {
        let zome_call = ZomeFnCall::new(&zome, cap, &fn_name, JsonString::from_json(&params));
        Ok(context.block_on(call_zome_function(Arc::clone(&context), zome_call))?)
    }

    /// call a function in a zome
    pub fn call_zome_function(
        context: Arc<Context>,
        zome: &str,
        cap: CapabilityRequest,
        fn_name: &str,
        params: &str,
    ) -> HolochainResult<JsonString> {
        let metric_name = format!("call_zome_function.{}.{}", zome, fn_name);
        with_latency_publishing!(
            metric_name,
            context.metric_publisher,
            Self::call_inner,
            context.clone(),
            zome,
            cap,
            fn_name,
            params
        )
    }

    /// checks to see if an instance is active
    pub fn active(&self) -> bool {
        self.active
    }

    /// return
    pub fn state(&self) -> Result<StateWrapper, HolochainInstanceError> {
        self.check_instance()?;
        Ok(self.instance.as_ref().unwrap().state())
    }

    pub fn context(&self) -> Result<Arc<Context>, HolochainInstanceError> {
        self.check_instance()?;
        Ok(self.context.as_ref().unwrap().clone())
    }

    pub fn set_conductor_api(&mut self, api: IoHandler) -> Result<(), HolochainInstanceError> {
        self.context()?.conductor_api.reset(api);
        Ok(())
    }

    pub fn get_state_dump(&self) -> Result<StateDump, HolochainInstanceError> {
        self.check_instance()?;
        Ok(StateDump::from(self.context.clone().expect(
            "Context must be Some since we've checked it with check_instance()? above",
        )))
    }

    pub fn get_type_and_content_from_cas(
        &self,
        address: &Address,
    ) -> Result<(String, String), HolochainInstanceError> {
        self.check_instance()?;
        Ok(address_to_content_and_type(
            address,
            self.context
                .clone()
                .expect("Context must be Some since we've checked it with check_instance()? above"),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use self::tempfile::tempdir;
    use super::*;
    use crate::context_builder::ContextBuilder;
    use holochain_core::{
        action::Action,
        context::Context,
        logger::{test_logger, TestLogger},
        nucleus::actions::call_zome_function::make_cap_request_for_call,
        signal::{signal_channel, SignalReceiver},
    };
    use holochain_core_types::dna::capabilities::CapabilityRequest;
    use holochain_json_api::json::RawString;
    use holochain_locksmith::Mutex;
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_core::wasm_engine::io::wasm_target_dir;
    use std::{path::PathBuf, sync::Arc};
    use tempfile;
    use test_utils::{
        create_arbitrary_test_dna, create_test_defs_with_fn_name, create_test_dna_with_defs,
        create_test_dna_with_wat, create_wasm_from_file, expect_action, hc_setup_and_call_zome_fn,
        mock_signing::{mock_conductor_api, registered_test_agent},
    };

    fn test_context(agent_name: &str) -> (Arc<Context>, Arc<Mutex<TestLogger>>, SignalReceiver) {
        let agent = registered_test_agent(agent_name);
        let (signal_tx, signal_rx) = signal_channel();
        let logger = test_logger();
        (
            Arc::new(
                ContextBuilder::new()
                    .with_agent(agent.clone())
                    .with_signals(signal_tx)
                    .with_conductor_api(mock_conductor_api(agent))
                    .with_file_storage(tempdir().unwrap().path().to_str().unwrap())
                    .unwrap()
                    .spawn(),
            ),
            logger,
            signal_rx,
        )
    }

    fn example_api_wasm_path() -> PathBuf {
        let mut path = wasm_target_dir(
            &String::from("conductor_lib").into(),
            &String::from("wasm-test").into(),
        );
        let wasm_path_component: PathBuf = [
            String::from("wasm32-unknown-unknown"),
            String::from("release"),
            String::from("example_api_wasm.wasm"),
        ]
        .iter()
        .collect();
        path.push(wasm_path_component);

        path
    }

    fn example_api_wasm() -> Vec<u8> {
        create_wasm_from_file(&example_api_wasm_path())
    }

    // for these tests we use the agent capability call
    fn cap_call(context: Arc<Context>, fn_name: &str, params: &str) -> CapabilityRequest {
        make_cap_request_for_call(
            context.clone(),
            Address::from(context.clone().agent_id.address()),
            fn_name,
            JsonString::from_json(params),
        )
    }

    #[test]
    fn can_instantiate() {
        let mut dna = create_arbitrary_test_dna();
        dna.name = "TestApp".to_string();
        let (context, _test_logger, _) = test_context("bob");
        let result = Holochain::new(dna.clone(), context.clone());
        assert!(result.is_ok());
        let hc = result.unwrap();
        let instance = hc.instance.as_ref().unwrap();
        let context = hc.context.as_ref().unwrap().clone();
        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert!(!hc.active);
        assert_eq!(context.agent_id.nick, "bob".to_string());
        let network_state = context.state().unwrap().network().clone();
        assert_eq!(network_state.agent_id.is_some(), true);
        assert_eq!(network_state.dna_address.is_some(), true);

        // This test is not meaningful anymore since the idiomatic logging refactoring
        // assert!(hc.instance.state().nucleus().has_initialized())
        // let _test_logger = test_logger.lock().unwrap();
        // assert!(format!("{:?}", *test_logger).contains("\"debug/conductor: TestApp instantiated\""));
    }

    #[test]
    fn can_persistant_and_load() {
        let temp = tempdir().unwrap();
        let temp_filestorage_dir = temp.path().to_str().unwrap();
        let agent = registered_test_agent("persister");
        let (signal_tx, _signal_rx) = signal_channel();
        let mut dna = create_arbitrary_test_dna();
        dna.name = "TestApp".to_string();

        {
            let context_new = Arc::new(
                ContextBuilder::new()
                    .with_agent(agent.clone())
                    .with_signals(signal_tx.clone())
                    .with_conductor_api(mock_conductor_api(agent.clone()))
                    .with_file_storage(temp_filestorage_dir)
                    .unwrap()
                    .spawn(),
            );

            let result = Holochain::new(dna.clone(), context_new.clone());
            assert!(result.is_ok());
            let hc = result.unwrap();
            let instance = hc.instance.as_ref().unwrap();
            let context = hc.context.as_ref().unwrap().clone();
            assert_eq!(instance.state().nucleus().dna(), Some(dna.clone()));
            assert!(!hc.active);
            assert_eq!(context.agent_id.nick, "persister".to_string());
            let network_state = context.state().unwrap().network().clone();
            assert_eq!(network_state.agent_id.is_some(), true);
            assert_eq!(network_state.dna_address.is_some(), true);
        }

        let context_load = Arc::new(
            ContextBuilder::new()
                .with_agent(agent.clone())
                .with_signals(signal_tx)
                .with_conductor_api(mock_conductor_api(agent.clone()))
                .with_file_storage(temp_filestorage_dir)
                .unwrap()
                .spawn(),
        );

        let result = Holochain::load(context_load.clone());
        if let Err(e) = result {
            panic!("Error during Holochain::load: {:?}", e);
        }
        assert!(result.is_ok());
        let hc = result.unwrap();
        let instance = hc.instance.as_ref().unwrap();
        let context = hc.context.as_ref().unwrap().clone();
        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert!(!hc.active);
        assert_eq!(context.agent_id.nick, "persister".to_string());
        let network_state = context.state().unwrap().network().clone();
        assert_eq!(network_state.agent_id.is_some(), true);
        assert_eq!(network_state.dna_address.is_some(), true);
    }

    #[test]
    fn fails_instantiate_if_init_fails() {
        let dna = create_test_dna_with_wat(
            "test_zome",
            Some(
                r#"
            (module
                (memory (;0;) 1)
                (func (export "init") (param $p0 i64) (result i64)
                    i64.const 9
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
            HolochainInstanceError::from(HolochainError::ErrorGeneric(
                "At least one zome init returned error: [(\"test_zome\", \"\\\"Init\\\"\")]"
                    .to_string()
            )),
            result.err().unwrap(),
        );
    }

    #[test]
    #[cfg(feature = "broken-tests")]
    fn fails_instantiate_if_init_times_out() {
        let dna = create_test_dna_with_wat(
            "test_zome",
            Callback::Init.capability().as_str(),
            Some(
                r#"
            (module
                (memory (;0;) 1)
                (func (export "init") (param $p0 i64) (result i64)
                    (loop (br 0))
                    i64.const 0
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
            HolochainInstanceError::from(HolochainError::ErrorGeneric(
                "Timeout while initializing".to_string()
            )),
            result.err().unwrap(),
        );
    }

    #[test]
    fn can_start_and_stop() {
        let dna = create_arbitrary_test_dna();
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
 (export "public_test_fn" (func $func0))
 (func $func0 (param $p0 i64) (result i64)
       i64.const 16
       )
 (data (i32.const 0)
       "{\"holo\":\"world\"}"
       )
 )
"#;
        let dna = create_test_dna_with_wat("test_zome", Some(wat));
        let (context, _, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

        let cap_call = cap_call(context.clone(), "public_test_fn", "");

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call,
            "public_test_fn",
            "",
        );
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from_json("{\"holo\":\"world\"}")
        );
    }

    #[test]
    fn can_get_state() {
        let dna = create_arbitrary_test_dna();
        let (context, _, _) = test_context("bob");
        let hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.state();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nucleus().dna(), Some(dna));
    }

    #[test]
    fn can_call_test() {
        let wasm = example_api_wasm();
        let defs = create_test_defs_with_fn_name("round_trip_test");
        let dna = create_test_dna_with_defs("test_zome", defs, &wasm);
        let (context, _, _) = test_context("bob");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

        hc.start().expect("couldn't start");

        let params = r#"{"input_int_val":2,"input_str_val":"fish"}"#;
        // always returns not implemented error for now!
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call(context.clone(), "round_trip_test", params),
            "round_trip_test",
            params,
        );
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from_json(
                r#"{"input_int_val_plus2":4,"input_str_val_plus_dog":"fish.puppy"}"#
            ),
        );
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_commit() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let defs = create_test_defs_with_fn_name("commit_test");
        let dna = create_test_dna_with_defs("test_zome", defs, &wasm);
        let (context, _, signal_rx) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

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
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call(context.clone(), "commit_test", r#"{}"#),
            "commit_test",
            r#"{}"#,
        );

        // Expect fail because no validation function in wasm
        assert!(result.is_ok(), "result = {:?}", result);
        // @TODO fragile test!
        assert_ne!(
            result.clone().ok().unwrap(),
            JsonString::from_json("{\"Err\":\"Argument deserialization failed\"}")
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
        let defs = create_test_defs_with_fn_name("commit_fail_test");
        let dna = create_test_dna_with_defs("test_zome", defs, &wasm);
        let (context, _, signal_rx) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");

        // Call the exposed wasm function that calls the Commit API function
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call(context.clone(), "commit_fail_test", r#"{}"#),
            "commit_fail_test",
            r#"{}"#,
        );
        println!("can_call_commit_err result: {:?}", result);

        // Expect normal OK result with hash
        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(
            result.ok().unwrap(),
            JsonString::from_json("{\"Err\":\"Argument deserialization failed\"}"),
        );

        expect_action(&signal_rx, |action| {
            if let Action::ReturnZomeFunctionResult(_) = action {
                true
            } else {
                false
            }
        })
        .unwrap();
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let defs = create_test_defs_with_fn_name("debug_hello");
        let dna = create_test_dna_with_defs("test_zome", defs, &wasm);

        let (context, _, signal_rx) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");

        // Call the exposed wasm function that calls the Commit API function
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call(context.clone(), "debug_hello", r#"{}"#),
            "debug_hello",
            r#"{}"#,
        );

        assert_eq!(Ok(JsonString::null()), result,);
        // @TODO https://github.com/holochain/holochain-rust/issues/928
        // let test_logger = test_logger.lock().unwrap();
        // assert!(format!("{:?}", test_logger.log).contains(
        //     "\"debug/dna: \\\'\\\"Hello world!\\\"\\\'\", \"debug/zome: Zome Function \\\'debug_hello\\\' returned: Success\""));

        expect_action(&signal_rx, |action| {
            if let Action::ReturnZomeFunctionResult(_) = action {
                true
            } else {
                false
            }
        })
        .unwrap();
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn can_call_debug_multiple() {
        // Setup the holochain instance
        let wasm = example_api_wasm();
        let defs = create_test_defs_with_fn_name("debug_multiple");
        let dna = create_test_dna_with_defs("test_zome", defs, &wasm);

        let (context, _, signal_rx) = test_context("alex");
        let mut hc = Holochain::new(dna.clone(), context.clone()).unwrap();

        // Run the holochain instance
        hc.start().expect("couldn't start");

        // Call the exposed wasm function that calls the Commit API function
        let result = Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            cap_call(context.clone(), "debug_multiple", r#"{}"#),
            "debug_multiple",
            r#"{}"#,
        );

        // Expect Success as result
        println!("result = {:?}", result);
        assert_eq!(Ok(JsonString::null()), result,);

        // @TODO https://github.com/holochain/holochain-rust/issues/928
        // let test_logger = test_logger.lock().unwrap();
        // assert!(format!("{:?}", test_logger.log).contains(
        //     "\"debug/dna: \\\'\\\"Hello\\\"\\\'\", \"debug/dna: \\\'\\\"world\\\"\\\'\", \"debug/dna: \\\'\\\"!\\\"\\\'\", \"debug/zome: Zome Function \\\'debug_multiple\\\' returned: Success\""));

        expect_action(&signal_rx, |action| {
            if let Action::ReturnZomeFunctionResult(_) = action {
                true
            } else {
                false
            }
        })
        .unwrap();
    }

    #[test]
    // TODO #165 - Move test to core/nucleus and use instance directly
    fn call_debug_stacked() {
        let call_result = hc_setup_and_call_zome_fn(
            &example_api_wasm_path(),
            "debug_stacked_hello",
            RawString::from(""),
        );
        assert_eq!(
            JsonString::from_json("{\"value\":\"fish\"}"),
            call_result.unwrap()
        );
    }

    #[test]
    #[cfg(feature = "broken-tests")] // breaks on windows.
    fn can_receive_action_signals() {
        use holochain_core::action::Action;
        use std::time::Duration;
        let wasm = include_bytes!(format!(
            "{}{slash}wasm32-unknown-unknown{slash}release{slash}example_api_wasm.wasm",
            slash = std::path::MAIN_SEPARATOR,
            wasm_target_dir("conductor_lib", "wasm-test"),
        ));
        let defs = test_utils::create_test_defs_with_fn_name("commit_test");
        let mut dna = test_utils::create_test_dna_with_defs("test_zome", defs, wasm);

        dna.uuid = "can_receive_action_signals".into();
        let (context, _, signal_rx) = test_context("alex");
        let timeout = 1000;
        let mut hc = Holochain::new(dna.clone(), context).unwrap();
        hc.start().expect("couldn't start");
        Holochain::call_zome_function(
            hc.context().unwrap(),
            "test_zome",
            example_capability_request(),
            "commit_test",
            r#"{}"#,
        )
        .unwrap();

        'outer: loop {
            let msg_publish = signal_rx
                .recv_timeout(Duration::from_millis(timeout))
                .expect("no more signals to receive (outer)");
            if let Signal::Trace(Action::Publish(address)) = msg_publish {
                loop {
                    let msg_hold = signal_rx
                        .recv_timeout(Duration::from_millis(timeout))
                        .expect("no more signals to receive (inner)");
                    if let Signal::Trace(Action::Hold(entry)) = msg_hold {
                        assert_eq!(address, entry.address());
                        break 'outer;
                    }
                }
            }
        }
    }
}
