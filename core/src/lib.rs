#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate multihash;
extern crate rust_base58;
extern crate serde;
extern crate serde_json;
extern crate snowflake;
#[cfg(test)]
extern crate test_utils;
extern crate wasmi;
#[macro_use]
extern crate bitflags;

extern crate holochain_agent;
extern crate holochain_dna;

pub mod agent;
pub mod chain;
pub mod context;
pub mod error;
pub mod hash;
pub mod hash_table;
pub mod instance;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod state;

#[cfg(test)]
mod tests {
    use super::*;
    use error::HolochainError;
    use holochain_dna::{zome::capabilities::ReservedCapabilityNames, Dna};
    use instance::Instance;
    use nucleus::{Action::*, FunctionCall};
    use state::{Action::*, State};
    use std::{sync::mpsc::channel, thread::sleep, time::Duration};
    use test_utils;

    /// create a test instance
    pub fn create_instance(dna: Dna) -> Instance {
        // Create instance and plug in our DNA
        let mut instance = Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Wait for Init to finish
        while instance.state().history.len() < 4 {
            // TODO - #21
            // This println! should be converted to either a call to the app logger, or to the core debug log.
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10))
        }

        instance
    }

    /// This test shows how to call dispatch with a closure that should run
    /// when the action results in a state change.  Note that the observer closure
    /// needs to return a boolean to indicate that it has successfully observed what
    /// it intends to observe.  It will keep getting called as the state changes until
    /// it returns true.
    /// Note also that for this test we create a channel to send something (in this case
    /// the dna) back over, just so that the test will block until the closure is successfully
    /// run and the assert will actually run.  If we put the assert inside the closure
    /// the test thread could complete before the closure was called.

    #[test]
    fn can_dispatch_with_observer() {
        let mut instance = Instance::new();
        instance.start_action_loop();

        let dna = Dna::new();
        let (sender, receiver) = channel();
        instance.dispatch_with_observer(
            Nucleus(InitApplication(dna.clone())),
            move |state: &State| match state.nucleus().dna() {
                Some(dna) => {
                    sender.send(dna).expect("test channel must be open");
                    return true;
                }
                None => return false,
            },
        );

        let stored_dna = receiver.recv().unwrap();

        assert_eq!(dna, stored_dna);
    }

    #[test]
    fn can_dispatch_and_wait() {
        let mut instance = Instance::new();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(
            instance.state().nucleus().status(),
            ::nucleus::NucleusStatus::New
        );

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();

        // the initial state is not intialized
        assert!(instance.state().nucleus().has_initialized() == false);

        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Wait for Init to finish
        while instance.state().history.len() < 2 {
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10));
        }
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    fn call_ribosome_function() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            "test_cap".to_string(),
            None,
        );
        let mut instance = create_instance(dna);

        // Create zome function call
        let call = FunctionCall::new("test_zome", "test_cap", "main", "");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);
        match result {
            // Result 1337 from WASM (as string)
            Ok(val) => assert_eq!(val, "1337"),
            Err(err) => assert_eq!(err, HolochainError::InstanceActive),
            //Err(_) => assert!(false),
        }
    }

    #[test]
    fn call_ribosome_wrong_dna() {
        let mut instance = Instance::new();
        instance.start_action_loop();

        let call = FunctionCall::new("test_zome", "test_cap", "main", "{}");
        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::DnaMissing) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn call_ribosome_wrong_function() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            "test_cap".to_string(),
            None,
        );
        let mut instance = create_instance(dna);

        // Create zome function call:
        let call = FunctionCall::new("test_zome", "test_cap", "xxx", "{}");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ErrorGeneric(err)) => {
                assert_eq!(err, "Function: Module doesn\'t have export xxx_dispatch")
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn call_wrong_ribosome_function() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            "test_cap".to_string(),
            None,
        );
        let mut instance = create_instance(dna);

        // Create bad zome function call
        let call = FunctionCall::new("xxx", "test_cap", "main", "{}");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ZomeNotFound(err)) => assert_eq!(err, "Zome 'xxx' not found"),
            _ => assert!(false),
        }

        // Create bad capability function call
        let call = FunctionCall::new("test_zome", "xxx", "main", "{}");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::CapabilityNotFound(err)) => {
                assert_eq!(err, "Capability 'xxx' not found in Zome 'test_zome'")
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_missing_genesis() {
        let mut dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            "test_cap".to_string(),
            None,
        );
        dna.zomes[0].capabilities[0].name = ReservedCapabilityNames::LifeCycle.as_str().to_string();

        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    fn test_genesis_ok() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                    i32.const 0
                )
                (data (i32.const 0)
                    ""
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    fn test_genesis_err() {
        let dna = test_utils::create_test_dna_with_wat(
            "test_zome".to_string(),
            ReservedCapabilityNames::LifeCycle.as_str().to_string(),
            Some(
                r#"
            (module
                (memory (;0;) 17)
                (func (export "genesis_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                    i32.const 4
                )
                (data (i32.const 0)
                    "1337"
                )
                (export "memory" (memory 0))
            )
        "#,
            ),
        );

        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized() == false);
    }

}
