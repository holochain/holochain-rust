#![feature(try_from)]
/// Holochain Container executable
///
/// This is (the beginnings) of the main container implementation maintained by the
/// Holochain Core team.
///
/// This executable will become what was referred to as the "pro" version of the container.
/// A GUI less background system service that manages multiple Holochain instances,
/// controlled through configuration files like [this example](container/example-config/basic.toml).
///
/// The interesting aspects of the container code is going on in [container_api](container_api).
/// This is mainly a thin wrapper around the structs and functions defined there.
///
/// If called without arguments, this executable tries to load a configuration from
/// ~/.holochain/container_config.toml.
/// A custom config can be provided with the --config, -c flag.
extern crate clap;
extern crate holochain_container_api;
extern crate holochain_core_types;
extern crate structopt;

use holochain_container_api::{
    config::{load_configuration, Configuration},
    container::{mount_container_from_config, CONTAINER},
};
use holochain_core_types::error::HolochainError;
use std::{fs::File, io::prelude::*, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "hcc")]
struct Opt {
    /// Output file
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let opt = Opt::from_args();
    let config_path = opt
        .config
        .unwrap_or(PathBuf::from(r"~/.holochain/container_config.toml"));
    let config_path_str = config_path.to_str().unwrap();
    println!("Using config path: {}", config_path_str);
    match bootstrap_from_config(config_path_str) {
        Ok(()) => {
            {
                let mut container_guard = CONTAINER.lock().unwrap();
                let mut container = container_guard.as_mut().expect("Container must be mounted");
                println!(
                    "Successfully loaded {} instance configurations",
                    container.instances().len()
                );
                println!("Starting all of them...");
                container
                    .start_all_instances()
                    .expect("Could not start instances!");
                println!("Starting interfaces...");
                container.start_all_interfaces();
                println!("Done.");
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
    mount_container_from_config(config);
    let mut container_guard = CONTAINER.lock().unwrap();
    let container = container_guard
        .as_mut()
        .expect("Container must be mounted");
    container.set_config_path(PathBuf::from(path));
    container.load_config()?;
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(load_configuration::<Configuration>(&contents)?)
}
