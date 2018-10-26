extern crate holochain_agent;
extern crate holochain_cas_implementations;
extern crate holochain_core;
extern crate holochain_core_api;
extern crate holochain_dna;
extern crate tempfile;

use holochain_agent::Agent;
use holochain_cas_implementations::{cas::file::FilesystemStorage, eav::file::EavFileStorage};
use holochain_core::{context::Context, logger::SimpleLogger, persister::SimplePersister};
use holochain_core_api::*;
use holochain_dna::Dna;
use std::{
    env,
    sync::{Arc, Mutex},
};

use tempfile::tempdir;

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn usage() {
    println!("Usage: holochain_test_bin <identity>");
    std::process::exit(1);
}

// this is all debug code, no need to track code test coverage
#[cfg_attr(tarpaulin, skip)]
fn main() {
    let tempdir = tempdir().unwrap();
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
    let agent = Agent::from(identity.to_string());
    let context = Context::new(
        agent,
        Arc::new(Mutex::new(SimpleLogger {})),
        Arc::new(Mutex::new(SimplePersister::new("foo".to_string()))),
        FilesystemStorage::new(tempdir.path().to_str().unwrap()).unwrap(),
        EavFileStorage::new(tempdir.path().to_str().unwrap().to_string()).unwrap(),
    ).expect("context is supposed to be created");
    let mut hc = Holochain::new(dna, Arc::new(context)).unwrap();
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
