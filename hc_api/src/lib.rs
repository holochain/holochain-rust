/*!
hc_api provides a library for container applications to instantiate and run holochain applications.

# Examples

``` rust
extern crate hc_core;
extern crate hc_api;
use hc_api::*;
use hc_core::nucleus::dna::*;

// instantiate a new app

// need to get to something like this:
//let dna = hc_dna::from_package_file("mydna.hcpkg");
//let agent = get_the_agent_somehow();
//let hc = App::new(dna,agent);
// but for now:
let dna = DNA {};
let mut hc = App::new(dna).expect("initialization failed");

// and then call a function
let call = hc.call("some_fn");

// ...

```
*/

extern crate hc_core;

/// App contains a Holochain application instance
pub struct App {
    instance: hc_core::instance::Instance,
}

use hc_core::error::HolochainError;
use hc_core::nucleus::dna::*;
use hc_core::nucleus::Action::*;
use hc_core::state::Action::*;
use hc_core::nucleus::fncall;

impl App {
    pub fn new(dna: DNA) -> Result<Self, HolochainError> {
        let mut instance = hc_core::instance::Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.dispatch(action);
        instance.consume_next_action()?;
        let app = App { instance: instance };
        Ok(app)
    }

    pub fn call(&mut self,fn_name:&str)  -> Result<(), HolochainError> {
        let call_data = fncall::Call::new(fn_name);
        let action = Nucleus(Call(call_data));
        self.instance.dispatch(action.clone());
        self.instance.consume_next_action()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_instantiate() {
        let dna = DNA {};
        let hc = App::new(dna.clone());
        match hc {
            Ok(app) => assert_eq!(app.instance.state().nucleus().dna(), Some(dna)),
            Err(_) => assert!(false),
        };
    }


    #[test]
    fn can_call() {
        let dna = DNA {};
        let mut hc = App::new(dna.clone()).unwrap();
        let call = hc.call("bogusfn");
        // allways returns not implemented error for now!
        match call {
            Ok(_) =>  assert!(false),
            Err(_) => assert!(true),
        };
    }
}
