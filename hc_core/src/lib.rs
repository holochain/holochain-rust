#![deny(warnings)]
extern crate hc_dna;
extern crate wabt;
pub mod agent;
pub mod common;
pub mod context;
pub mod error;
pub mod instance;
pub mod logger;
pub mod network;
pub mod nucleus;
pub mod persister;
pub mod source_chain;
pub mod state;

#[cfg(test)]
mod tests {
    //use agent::Action::*;
    use hc_dna::wasm::DnaWasm;
    use hc_dna::zome::capabilities::Capability;
    use hc_dna::zome::Zome;
    use hc_dna::Dna;
    use instance::Instance;
    use nucleus::Action::*;
    use nucleus::FunctionCall;
    use state::Action::*;
    use state::State;
    use std::sync::mpsc::channel;
    use wabt::Wat2Wasm;
    use super::*;

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
        assert_eq!(instance.state().nucleus().initialized(), false);

        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());

        assert_eq!(instance.state().nucleus().dna(), Some(dna));
        assert_eq!(instance.state().nucleus().initialized(), true);

        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().initialized(), true);
    }

    #[test]
    fn call_ribosome_function() {
        // Test WASM code that returns 1337 as integer
        let wasm_binary = Wat2Wasm::new()
            .canonicalize_lebs(false)
            .write_debug_names(true)
            .convert(
                r#"
                (module
                    (memory (;0;) 17)
                    (func (export "main") (result i32)
                        i32.const 1337
                    )
                    (export "memory" (memory 0))
                )
            "#,
            )
            .unwrap();

        // Prepare valid DNA struct with that WASM in a zome's capability:
        let mut dna = Dna::new();
        let mut zome = Zome::new();
        let mut capability = Capability::new();
        capability.name = "test_cap".to_string();
        capability.code = DnaWasm {
            code: wasm_binary.as_ref().to_vec(),
        };
        zome.name = "test_zome".to_string();
        zome.capabilities.push(capability);
        dna.zomes.push(zome);

        // Create instance and plug in our DNA:
        let mut instance = Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.start_action_loop();
        instance.dispatch_and_wait(action.clone());
        assert_eq!(instance.state().nucleus().dna(), Some(dna));

        // Create zome function call:
        let call = FunctionCall::new(
            "test_zome",
            "test_cap",
            "main",
            "{}",
        );

        let result = nucleus::call_and_wait_for_result(call, &mut instance);

        // Result 1337 from WASM (as string)
        assert_eq!(result, "1337")
    }
}
