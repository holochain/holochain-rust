extern crate hc_dna;
use hc_dna::Dna;

pub mod fncall;
pub mod ribosome;

use error::HolochainError;
//use self::ribosome::*;
use state;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct NucleusState {
    dna: Option<Dna>,
    initialized: bool,
}

impl NucleusState {
    pub fn new() -> Self {
        NucleusState {
            dna: None,
            initialized: false,
        }
    }

    pub fn dna(&self) -> Option<Dna> {
        self.dna.clone()
    }

    pub fn initialized(&self) -> bool {
        self.initialized
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    InitApplication(Dna),
    Call(fncall::Call),
}

pub fn reduce(
    old_state: Rc<NucleusState>,
    action: &state::Action,
) -> Result<Rc<NucleusState>, HolochainError> {
    match *action {
        state::Action::Nucleus(ref nucleus_action) => {
            let mut new_state: NucleusState = (*old_state).clone();
            match *nucleus_action {
                Action::InitApplication(ref dna) => {
                    if new_state.initialized {
                        return Err(HolochainError::AllreadyInitialized);
                    }
                    new_state.dna = Some(dna.clone());
                    new_state.initialized = true;
                }
                Action::Call(_) => return Err(HolochainError::NotImplemented),
            }
            Ok(Rc::new(new_state))
        }
        _ => Ok(old_state),
    }
}

#[cfg(test)]
mod tests {
    use super::super::nucleus::Action::*;
    use super::super::state::Action::*;
    use super::*;

    #[test]
    fn can_instantiate_nucleus_state() {
        let state = NucleusState::new();
        assert_eq!(state.dna, None);
        assert_eq!(state.initialized, false);
    }

    #[test]
    fn can_reduce_initialize_action() {
        let state = NucleusState::new();
        let dna = Dna::new();
        let action = Nucleus(InitApplication(dna));
        let mut new_state = Rc::new(NucleusState::new()); // initialize to bogus value
        match reduce(Rc::new(state), &action) {
            Ok(state) => {
                new_state = state;
                assert!(new_state.initialized, true)
            }
            Err(_) => assert!(false),
        };

        // on second reduction it should throw error
        match reduce(new_state, &action) {
            Ok(_) => assert!(false),
            Err(err) => match err {
                HolochainError::AllreadyInitialized => assert!(true),
                _ => assert!(false),
            },
        };
    }

    #[test]
    fn can_reduce_call_action() {
        let state = NucleusState::new();
        let call = fncall::Call::new("bogusfn");
        let action = Nucleus(Call(call));
        match reduce(Rc::new(state), &action) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        };
    }
}
