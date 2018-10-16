extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_core_types;

use holochain_core::{context::Context, logger::SimpleLogger, persister::SimplePersister};
use holochain_core_api::*;
use holochain_core_types::entry::Entry;
use std::{
    env,
    sync::{Arc, Mutex},
};

use holochain_core_types::entry::{agent::AgentId, dna::Dna};

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: holochain_test_bin <identity>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    // let identity = &args[1];
    //
    // if identity == "" {
    //     usage();
    // }

    //let dna = holochain_dna::from_package_file("mydna.hcpkg");
    let dna = Dna::new();
    let agent_id = Entry::AgentId(AgentId::default());
    let context = Context::new(
        &agent_id,
        Arc::new(Mutex::new(SimpleLogger {})),
        Arc::new(Mutex::new(SimplePersister::new())),
    );
    let mut hc = Holochain::new(dna, Arc::new(context)).unwrap();
    println!("Created a new instance with agent id: {}", agent_id);

    // start up the app
    hc.start().expect("couldn't start the app");
    println!("Started the app..");

    // call a function in the app
    //hc.call("some_fn");

    // get the state
    {
        let state = hc.state().unwrap();
        println!("Agent State: {:?}", state.agent());

        // do some other stuff with the state here
        // ...
    }

    // stop the app
    hc.stop().expect("couldn't stop the app");
    println!("Stopped the app..");
}
