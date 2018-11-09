#![feature(try_from)]
extern crate clap;
extern crate holochain_container_api;
extern crate holochain_core_types;

use clap::{Arg, App};
use holochain_core_types::error::HolochainError;
use holochain_container_api::{
    config::{
        Configuration, load_configuration,
    },
    container::Container,
};
use std::{convert::TryFrom, fs::File, io::prelude::*};

fn main() {
    let matches = App::new("hcc")
        .version("0.0.1")
        .author("Holochain Core Dev Team <devcore@holochain.org>")
        .about("Headless Holochain Container Service")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .get_matches();
    let config_path = matches.value_of("config").unwrap_or("~/.holochain/container_config.toml");
    println!("Using config path: {}", config_path);
    match bootstrap_from_config(config_path) {
        Ok(mut container) => {
            if container.instances.len() > 0 {
                println!("Successfully loaded {} instance configurations", container.instances.len());
                println!("Starting all of them...");
                container.start_all();
                println!("Done.");
                loop {};
            } else {
                println!("No instance started, bailing...");
            }
        },
        Err(error) => println!("Error while trying to boot from config: {:?}", error),
    };
}

fn bootstrap_from_config(path: &str) -> Result<Container, HolochainError> {
    let config = load_config_file(&String::from(path))?;
    config.check_consistency().map_err(|string| HolochainError::ConfigError(string))?;
    Container::try_from(&config)
}

fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(load_configuration::<Configuration>(&contents)?)
}

