#![cfg_attr(feature = "strict", deny(warnings))]

extern crate hc_dna;

use hc_dna::Dna;

extern crate hc_core;

use hc_core::common;

use hc_core::agent::Action::*;
use hc_core::instance::Instance;
use hc_core::nucleus::Action::*;
use hc_core::state::Action::*;

fn main() {
    println!("Creating instance..");
    let mut instance = Instance::new();

    let dna = Dna::new();
    println!("adding action: {:?}", InitApplication(dna));
    let dna = Dna::new();
    instance.dispatch(Nucleus(InitApplication(dna)));
    println!("pending actions: {:?}", instance.pending_actions());

    let entry = common::entry::Entry {};
    let action = Agent(Commit(entry));
    println!("adding action: {:?}", action);
    instance.dispatch(action);
    println!("pending actions: {:?}", instance.pending_actions());

    let dna = Dna::new();
    instance.dispatch(Nucleus(InitApplication(dna)));

    println!("consuming action...");
    instance.consume_next_action().expect("consume failed");
    println!("pending actions: {:?}", instance.pending_actions());

    println!("consuming action...");
    instance.consume_next_action().expect("consume failed");
    println!("pending actions: {:?}", instance.pending_actions());
    instance.consume_next_action().expect("consume failed");
}
