#![feature(try_from)]
/// Holochain Conductor executable
///
/// This is (the beginnings) of the main conductor implementation maintained by the
/// Holochain Core team.
///
/// This executable will become what was referred to as the "pro" version of the conductor.
/// A GUI less background system service that manages multiple Holochain instances,
/// controlled through configuration files like [this example](conductor/example-config/basic.toml).
///
/// The interesting aspects of the conductor code is going on in [conductor_api](conductor_api).
/// This is mainly a thin wrapper around the structs and functions defined there.
///
/// If called without arguments, this executable tries to load a configuration from
/// ~/.holochain/conductor/conductor_config.toml.
/// A custom config can be provided with the --config, -c flag.
extern crate clap;
extern crate holochain_conductor_api;
extern crate holochain_core_types;
extern crate structopt;

use holochain_conductor_api::{
    conductor::{mount_conductor_from_config, CONDUCTOR},
    config::{self, load_configuration, Configuration},
};
use holochain_core_types::error::HolochainError;
use std::{fs::File, io::prelude::*, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "holochain")]
struct Opt {
    /// Path to the toml configuration file for the conductor
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let opt = Opt::from_args();
    let config_path = opt
        .config
        .unwrap_or(config::default_persistence_dir().join("conductor-config.toml"));
    let config_path_str = config_path.to_str().unwrap();
    println!("Using config path: {}", config_path_str);
    match bootstrap_from_config(config_path_str) {
        Ok(()) => {
            {
                let mut conductor_guard = CONDUCTOR.lock().unwrap();
                let mut conductor = conductor_guard.as_mut().expect("Conductor must be mounted");
                println!(
                    "Successfully loaded {} instance configurations",
                    conductor.instances().len()
                );
                println!("Starting all of them...");
                conductor
                    .start_all_instances()
                    .expect("Could not start instances!");
                println!("Starting interfaces...");
                conductor.start_all_interfaces();
                println!("Done.");
                println!("Starting UI servers");
                conductor
                    .start_all_static_servers()
                    .expect("Could not start UI servers!");
            }
            loop {}
        }
        Err(error) => println!("Error while trying to boot from config: {:?}", error),
    };
}

#[cfg_attr(tarpaulin, skip)]
fn bootstrap_from_config(path: &str) -> Result<(), HolochainError> {
    let config = load_config_file(&String::from(path))?;
    config
        .check_consistency()
        .map_err(|string| HolochainError::ConfigError(string))?;
    mount_conductor_from_config(config);
    let mut conductor_guard = CONDUCTOR.lock().unwrap();
    let conductor = conductor_guard.as_mut().expect("Conductor must be mounted");
    conductor.load_config()?;
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(load_configuration::<Configuration>(&contents)?)
}