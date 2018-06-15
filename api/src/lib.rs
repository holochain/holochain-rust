extern crate hc_core;

/// App contains a Holochain application instance
pub struct App {
    instance: hc_core::instance::Instance,
}

use hc_core::error::HolochainError;
use hc_core::nucleus::dna::*;
use hc_core::nucleus::Action::*;
use hc_core::state::Action::*;
//use hc_core::nucleus::fncall;

impl App {
    pub fn new(dna: DNA) -> Result<Self, HolochainError> {
        let mut instance = hc_core::instance::Instance::new();
        let action = Nucleus(InitApplication(dna.clone()));
        instance.dispatch(action);
        instance.consume_next_action()?;
        let app = App { instance: instance };
        Ok(app)
    }

    /* not working yet
    pub fn call(&self,fn_name:&str) {
        let call_data = fncall::Call::new(fn_name);
        let action = Nucleus(Call(call_data));
        self.instance.dispatch(action.clone())
    }*/
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

    /*
    #[test]
    fn can_call() {
        let dna = DNA {};
        let hc = App::new(dna.clone()).unwrap();
        let call = fncall::Call::new("bogusfn");


    }*/
}
