pub mod dna;
pub mod ribosome;

use self::dna::DNA;
//use self::ribosome::*;
use state;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct NucleusState {
    dna: Option<DNA>,
    inits: i32
}

impl NucleusState {
    pub fn create() -> Self {
        NucleusState {
            dna: None,
            inits: 0
        }
    }

    pub fn dna(&self) -> Option<DNA> {
        self.dna.clone()
    }

    pub fn inits(&self) -> i32 {
        self.inits
    }

}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    InitApplication(DNA)
}



pub fn reduce(old_state: Rc<NucleusState>, action: &state::Action) -> Rc<NucleusState> {
    match *action {
        state::Action::Nucleus(ref nucleus_action) => {
            let mut new_state: NucleusState = (*old_state).clone();
            match *nucleus_action {
                Action::InitApplication(ref dna) => {
                    new_state.dna = Some(dna.clone());
                    new_state.inits += 1;
                    println!("DNA initialized: {}", new_state.inits)
                }
            }
            Rc::new(new_state)
        },
        _ => old_state
    }
}
