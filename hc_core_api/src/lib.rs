/*!
hc_core_api provides a library for container applications to instantiate and run holochain applications.

# Examples

``` rust
extern crate hc_core;
extern crate hc_core_api;
extern crate hc_dna;
extern crate hc_agent;

use hc_core_api::*;
use hc_dna::Dna;
use hc_agent::Agent;
use std::sync::{Arc, Mutex};
use hc_core::context::Context;
use hc_core::logger::SimpleLogger;
use hc_core::persister::SimplePersister;

// instantiate a new app

// need to get to something like this:
//let dna = hc_dna::from_package_file("mydna.hcpkg");

// but for now:
let dna = Dna::new();
let agent = Agent::from_string("bob");
let context = Context {
    agent: agent,
    logger: Arc::new(Mutex::new(SimpleLogger {})),
    persister: Arc::new(Mutex::new(SimplePersister::new())),
};
let mut hc = Holochain::new(dna,Arc::new(context)).unwrap();

// start up the app
hc.start().expect("couldn't start the app");

// call a function in the app
hc.call("test_zome","test_cap","some_fn","{}");

// get the state
{
    let state = hc.state();

    // do some other stuff with the state here
    // ...
}

// stop the app
hc.stop().expect("couldn't stop the app");

```
*/

extern crate hc_agent;
extern crate hc_core;
extern crate hc_dna;

use hc_core::context::Context;
use hc_dna::Dna;
use std::sync::Arc;

/// contains a Holochain application instance
pub struct Holochain {
    instance: hc_core::instance::Instance,
    #[allow(dead_code)]
    context: Arc<hc_core::context::Context>,
    active: bool,
}

use hc_core::error::HolochainError;
use hc_core::nucleus::Action::*;
use hc_core::nucleus::{call_and_wait_for_result, FunctionCall};
use hc_core::state::Action::*;
use hc_core::state::State;

impl Holochain {
    /// create a new Holochain instance
    pub fn new(dna: Dna, context: Arc<Context>) -> Result<Self, HolochainError> {
        let mut instance = hc_core::instance::Instance::new();
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

        let call = FunctionCall::new(zome, cap, fn_name, params);

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
    use hc_agent::Agent as HCAgent;
    use hc_core::context::Context;
    use hc_core::logger::Logger;
    use hc_core::persister::SimplePersister;
    use hc_core::test_utils::create_test_dna_with_wasm;
    use std::fmt;
    use std::sync::{Arc, Mutex};

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

    fn test_context(agent: hc_agent::Agent) -> (Arc<Context>, Arc<Mutex<TestLogger>>) {
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
        let wasm = r#"
(module
 (memory 1)
 (export "memory" (memory 0))
 (export "hello" (func $func0))
 (func $func0 (result i32)
       i32.const 16
       )
 (data (i32.const 256)
       "{\"holo\":\"world\"}"
       )
 )
"#;
        let dna = create_test_dna_with_wasm(Some(wasm));
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        let result = hc.call("test_zome", "test_cap", "hello", "{}");
        match result {
            Err(HolochainError::InstanceNotActive) => assert!(true),
            Err(_) => assert!(false),
            Ok(_) => assert!(false),
        }

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call("test_zome", "test_cap", "hello", "{}");

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
        let wasm = r#"
(module
  (type $t0 (func (param i32 i32) (result i32)))
  (type $t1 (func (param i32)))
  (type $t2 (func))
  (import "env" "print" (func $print (type $t1)))
  (func $_call (export "_call") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
    (local $l0 i32)
    (call $print
      (get_local $p1))
    (block $B0
      (br_if $B0
        (i32.eqz
          (get_local $p1)))
      (set_local $l0
        (i32.const 0))
      (loop $L1
        (call $print
          (i32.load8_u
            (i32.add
              (get_local $p0)
              (get_local $l0))))
        (br_if $L1
          (i32.lt_u
            (tee_local $l0
              (i32.add
                (get_local $l0)
                (i32.const 1)))
            (get_local $p1)))))
    (i32.const 0))
  (func $test (export "test") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
    (i32.store8 offset=2
      (get_local $p0)
      (i32.const 31))
    (i32.const 5))
  (func $rust_eh_personality (export "rust_eh_personality") (type $t2))
  (table $T0 1 1 anyfunc)
  (memory $memory (export "memory") 17)
  (global $g0 (mut i32) (i32.const 1049600)))
"#;
        let dna = create_test_dna_with_wasm(Some(wasm));
        let agent = HCAgent::from_string("bob");
        let (context, _) = test_context(agent.clone());
        let mut hc = Holochain::new(dna.clone(), context).unwrap();

        hc.start().expect("couldn't start");

        // always returns not implemented error for now!
        let result = hc.call("test_zome", "test_cap", "test", "{}");

        match result {
            Ok(result) => assert_eq!(result, "{\"holo\":\"world\"}"),
            Err(_) => assert!(false),
        };
    }

}
