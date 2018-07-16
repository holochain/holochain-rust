extern crate holochain_agent;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_dna;

use holochain_agent::Agent;
use holochain_core::{context::Context, logger::SimpleLogger, persister::SimplePersister};
use holochain_core_api::*;
use holochain_dna::Dna;
use std::{
    env, sync::{Arc, Mutex},
};

fn usage() {
    println!("Usage: holochain_test_bin <identity>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
    }

    let identity = &args[1];

    if identity == "" {
        usage();
    }

    //let dna = holochain_dna::from_package_file("mydna.hcpkg");
    let dna = Dna::new();
    let agent = Agent::from_string(identity);
    let context = Context {
        agent,
        logger: Arc::new(Mutex::new(SimpleLogger {})),
        persister: Arc::new(Mutex::new(SimplePersister::new())),
    };
    let mut hc = Holochain::new(dna, Arc::new(context)).unwrap();
    println!("Created a new instance with identity: {}", identity);

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
