#![cfg_attr(feature = "strict", deny(warnings))]

use hc_core::common;
use hc_core::nucleus;

use hc_core::agent::Action::*;
use hc_core::instance::Instance;
use hc_core::nucleus::Action::*;
use hc_core::state::Action::*;

fn main() {
    println!("Creating instance..");
    let mut instance = Instance::create();

    let dna = nucleus::dna::DNA {};
    println!("adding action: {:?}", InitApplication(dna));
    let dna = nucleus::dna::DNA {};
    instance.dispatch(Nucleus(InitApplication(dna)));
    println!("pending actions: {:?}", instance.pending_actions());

    let entry = common::entry::Entry::new(&String::new());
    let action = Agent(Commit(entry));
    println!("adding action: {:?}", action);
    instance.dispatch(action);
    println!("pending actions: {:?}", instance.pending_actions());

    let dna = nucleus::dna::DNA {};
    instance.dispatch(Nucleus(InitApplication(dna)));

    println!("consuming action...");
    instance.consume_next_action();
    println!("pending actions: {:?}", instance.pending_actions());

    println!("consuming action...");
    instance.consume_next_action();
    println!("pending actions: {:?}", instance.pending_actions());
    instance.consume_next_action();
}
