#![deny(warnings)]

#[macro_use]
extern crate serde_derive;
extern crate holochain_dna;
extern crate serde;
extern crate serde_json;
extern crate wabt;

pub mod agent;
pub mod chain;
pub mod common;
pub mod context;
pub mod error;
pub mod instance;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod state;

//#[cfg(test)]
pub mod test_utils {
    use super::*;
    use holochain_dna::wasm::DnaWasm;
    use holochain_dna::zome::capabilities::Capability;
    use holochain_dna::zome::capabilities::ReservedCapabilityNames;
    use holochain_dna::zome::Zome;
    use holochain_dna::Dna;
    use wabt::Wat2Wasm;

    use std::fs::File;

    pub fn create_wasm_from_file(fname: &str) -> Vec<u8> {
        use std::io::prelude::*;
        let mut file = File::open(fname).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        buf
    }

    pub fn create_test_dna_with_wat(zome_name : String, cap_name : String, wat: Option<&str>) -> Dna {
        // Default WASM code returns 1337 as integer
        let default_wat = format!(
            r#"
                (module
                    (memory (;0;) 17)
                    (func (export "main_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                        i32.const 4
                    )
                    (data (i32.const {})
                        "1337"
                    )
                    (export "memory" (memory 0))
                )
            "#,
            nucleus::ribosome::RESULT_OFFSET
        );
        let wat_str = match wat {
            None => default_wat.as_str(),
            Some(w) => w,
        };

        let wasm_binary = Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(wat_str)
            .unwrap();

        return create_test_dna_with_wasm(zome_name, cap_name, wasm_binary.as_ref().to_vec());
    }


    // Prepare valid DNA struct with that WASM in a zome's capability
    pub fn create_test_dna_with_wasm(zome_name : String, cap_name : String, wasm: Vec<u8>) -> Dna {
        let mut dna = Dna::new();
        let mut zome = Zome::new();
        let mut capability = Capability::new();
        capability.name = cap_name.to_string();
        capability.code = DnaWasm { code: wasm };
        zome.name = zome_name.to_string();
        zome.capabilities.push(capability);
        dna.zomes.push(zome);
        dna
    }


    // Prepare valid DNA struct helper
//    pub fn create_dna(zome_name : String, cap_name : String, wasm_binary : WabtBuf) -> Dna {
//        let mut dna = Dna::new();
//        let mut zome = Zome::new();
//        let mut capability = Capability::new();
//        capability.name = cap_name;
//        capability.code = DnaWasm { code: wasm_binary.as_ref().to_vec() };
//        zome.name = zome_name.to_string();
//        zome.capabilities.push(capability);
//        dna.zomes.push(zome);
//        dna
//    }


    // Create DNA containing WASM code with genesis that returns 0
    pub fn create_dna_with_genesis_ok() -> Dna {
        let wat =
            r#"
                (module
                    (memory (;0;) 17)
                    (func (export "genesis_dispatch") (param $p0 i32) (param $p1 i32) (result i32)
                        i32.const 1
                    )
                    (data (i32.const 0)
                        "0"
                    )
                    (export "memory" (memory 0))
                )
            "#;
        return create_test_dna_with_wat("test_zome".to_string(), ReservedCapabilityNames::LifeCycle.as_str().to_string(), Some(wat));
    }


    // Create DNA containing WASM code with genesis that returns 1337
    pub fn create_dna_with_genesis_err() -> Dna {
        let wat =
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
            "#;
        return create_test_dna_with_wat("test_zome".to_string(), ReservedCapabilityNames::LifeCycle.as_str().to_string(), Some(wat));
    }
}


#[cfg(test)]
mod tests {
    //use agent::Action::*;
    use super::*;
    use error::HolochainError;
    use holochain_dna::Dna;
    use instance::Instance;
    use nucleus::Action::*;
    use nucleus::FunctionCall;
    use state::Action::*;
    use state::State;
    use std::sync::mpsc::channel;

    use std::thread::sleep;
    use std::time::Duration;

    use holochain_dna::zome::capabilities::ReservedCapabilityNames;

    // This test shows how to call dispatch with a closure that should run
    // when the action results in a state change.  Note that the observer closure
    // needs to return a boolean to indicate that it has successfully observed what
    // it intends to observe.  It will keep getting called as the state changes until
    // it returns true.
    // Note also that for this test we create a channel to send something (in this case
    // the dna) back over, just so that the test will block until the closure is successfully
    // run and the assert will actually run.  If we put the assert inside the closure
    // the test thread could complete before the closure was called.
    #[test]
    fn dispatch_with_observer() {
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
    fn dispatch_and_wait() {
        let mut instance = Instance::new();
        assert_eq!(instance.state().nucleus().dna(), None);
        assert_eq!(instance.state().nucleus().status(), ::nucleus::NucleusStatus::New);

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert!(instance.state().nucleus().has_initialized() == false);

        // Wait for Init to finish
        while instance.state().history.len() < 2 {
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10));
        }
        assert!(instance.state().nucleus().has_initialized());
    }

    fn create_instance(dna: Dna) -> Instance {
        // Create instance and plug in our DNA
        let mut instance = Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Wait for Init to finish
        while instance.state().history.len() < 4 {
            println!("Waiting... {}", instance.state().history.len());
            sleep(Duration::from_millis(10))
        }

        instance
    }

    #[test]
    fn call_ribosome_function() {
        let dna = test_utils::create_test_dna_with_wat("test_zome".to_string(), "test_cap".to_string(),None);
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
        let dna = test_utils::create_test_dna_with_wat("test_zome".to_string(), "test_cap".to_string(),None);
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
        let dna = test_utils::create_test_dna_with_wat("test_zome".to_string(), "test_cap".to_string(),None);
        let mut instance = create_instance(dna);


        // Create bad zome function call
        let call = FunctionCall::new("xxx", "test_cap", "main", "{}");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::ZomeNotFound(err)) => {
                assert_eq!(err, "Zome 'xxx' not found")
            }
            _ => assert!(false),
        }

        // Create bad capability function call
        let call = FunctionCall::new("test_zome", "xxx", "main", "{}");

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        match result {
            Err(HolochainError::CapabilityNotFound(err)) => {
                assert_eq!(err, "Capability 'xxx' not found in Zome 'test_zome'")
            }
            _ => { assert!(false) },
        }
    }

    #[test]
    fn test_missing_genesis() {
        let mut dna = test_utils::create_test_dna_with_wat("test_zome".to_string(), "test_cap".to_string(),None);
        dna.zomes[0].capabilities[0].name = ReservedCapabilityNames::LifeCycle.as_str().to_string();

        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    fn test_genesis_ok() {
        let dna = test_utils::create_dna_with_genesis_ok();
        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized());
    }

    #[test]
    fn test_genesis_err() {
        let dna = test_utils::create_dna_with_genesis_err();
        let instance = create_instance(dna);

        assert_eq!(instance.state().history.len(), 4);
        assert!(instance.state().nucleus().has_initialized() == false);
    }


}
