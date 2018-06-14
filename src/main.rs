pub mod agent;
mod common;
mod network;
mod nucleus;
pub mod state;
pub mod instance;

use instance::Instance;
use state::Action::*;
use nucleus::Action::*;
use agent::Action::*;

fn main() {
    println!("Creating instance..");
    let mut instance = Instance::create();

    let dna = nucleus::dna::DNA{};
    println!("adding action: {:?}", InitApplication(dna));
    let dna = nucleus::dna::DNA{};
    instance.dispatch(Nucleus(InitApplication(dna)));
    println!("pending actions: {:?}", instance.pending_actions());

    let entry = common::entry::Entry{};
    let action = Agent(Commit(entry));
    println!("adding action: {:?}", action);
    instance.dispatch(action);
    println!("pending actions: {:?}", instance.pending_actions());

    let dna = nucleus::dna::DNA{};
    instance.dispatch(Nucleus(InitApplication(dna)));

    println!("consuming action...");
    instance.consume_next_action();
    println!("pending actions: {:?}", instance.pending_actions());

    println!("consuming action...");
    instance.consume_next_action();
    println!("pending actions: {:?}", instance.pending_actions());
    instance.consume_next_action();
}
