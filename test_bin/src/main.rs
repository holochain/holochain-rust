extern crate holochain_cas_implementations;
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;
extern crate serde_json;
extern crate tempfile;
#[macro_use]
extern crate failure;

use failure::Error;

use holochain_container_api::{context_builder::ContextBuilder, *};
use holochain_core_types::{agent::AgentId, dna::Dna};
use std::{env, sync::Arc};

use tempfile::tempdir;

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: holochain_test_bin <identity>");
    std::process::exit(1);
}

/// Network Test executable entry point.
// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    // Check args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    let identity = &args[1];
    if identity == "" {
        usage();
    }

    // Create Context for Holochain Core
    // let dna = holochain_core_types::dna::from_package_file("mydna.hcpkg");
    let tempdir = tempdir().unwrap();
    let dna = Dna::new();
    let agent = AgentId::generate_fake(identity);
    let context = ContextBuilder::new()
        .with_agent(agent)
        .with_file_storage(tempdir.path().to_str().unwrap())
        .expect("Tempdir must be accessible")
        .spawn();

    // Create Holochain Instance
    let mut hc =
        Holochain::new(dna, Arc::new(context)).expect("Holochain instance creation failed.");
    println!("Created a new instance with identity: {}", identity);

    // start up the holochain instance
    hc.start().expect("couldn't start the holochain instance");
    println!("Started the holochain instance..");

    // call a function in the zome code
    //hc.call("some_fn");

    // get the state
    {
        let state = hc.state().unwrap();
        println!("Agent State: {:?}", state.agent());

        // do some other stuff with the state here
        // ...
    }

    // stop the holochain instance
    hc.stop().expect("couldn't stop the holochain instance");
    println!("Stopped the holochain instance..");
}
