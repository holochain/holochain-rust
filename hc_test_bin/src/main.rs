#![cfg_attr(feature = "strict", deny(warnings))]

extern crate hc_agent;
extern crate hc_core;
extern crate hc_core_api;
extern crate hc_dna;

use hc_agent::Agent;
use hc_core::context::Context;
use hc_core::logger::SimpleLogger;
use hc_core::persister::SimplePersister;
use hc_core_api::*;
use hc_dna::Dna;
use std::sync::{Arc, Mutex};

use std::env;

fn usage() {
    println!("Usage: hc_test_bin <identity>");
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

    //let dna = hc_dna::from_package_file("mydna.hcpkg");
    let dna = Dna::new();
    let agent = Agent::from_string(identity);
    let context = Context {
        agent: agent,
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
