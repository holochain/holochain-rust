#![warn(unused_extern_crates)]
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
extern crate crossbeam_channel;
extern crate holochain_conductor_lib;
extern crate holochain_core_types;
extern crate holochain_locksmith;
extern crate holochain_tracing;
extern crate lib3h_sodium;
#[cfg(unix)]
extern crate signal_hook;
extern crate structopt;
#[macro_use]
extern crate log;

use holochain_conductor_lib::{
    conductor::{mount_conductor_from_config, Conductor, CONDUCTOR},
    config::{self, load_configuration, Configuration},
};
use holochain_core_types::{
    error::HolochainError, hdk_version::HDK_VERSION, BUILD_DATE, GIT_BRANCH, GIT_HASH, HDK_HASH,
};
use holochain_locksmith::spawn_locksmith_guard_watcher;
use holochain_tracing as ht;
#[cfg(unix)]
use signal_hook::{iterator::Signals, SIGINT, SIGTERM};
use std::{fs::File, io::prelude::*, path::PathBuf, sync::Arc, thread};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "holochain")]
struct Opt {
    /// Path to the toml configuration file for the conductor
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,
    #[structopt(short = "i", long = "info")]
    info: bool,
}

pub enum SignalConfiguration {
    Unix,
    Windows,
}

impl Default for SignalConfiguration {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            SignalConfiguration::Windows
        } else {
            SignalConfiguration::Unix
        }
    }
}

// NOTE: don't change without also changing in crates/trycp_server/src/main.rs
const MAGIC_STRING: &str = "*** Done. All interfaces started.";

#[cfg_attr(tarpaulin, skip)]
fn main() {
    // Tracer config:
    let tracer = {
        let (span_tx, span_rx) = crossbeam_channel::unbounded();

        let _ = thread::Builder::new()
            .name("tracer_loop".to_string())
            .spawn(move || {
                info!("Tracer loop started.");
                // TODO: killswitch
                let reporter = ht::Reporter::new("Holochain Core").unwrap();
                for span in span_rx {
                    reporter.report(&[span]).expect("could not report span");
                }
            });
        ht::Tracer::with_sender(ht::AllSampler, span_tx)
    };

    let _trace_guard = ht::push_span(tracer.span("holochain::main").start().into());

    let _ = spawn_locksmith_guard_watcher();

    lib3h_sodium::check_init();
    let opt = Opt::from_args();

    if opt.info {
        println!(
            "HDK_VERSION: {}\nHDK_HASH: {}",
            HDK_VERSION.to_string(),
            HDK_HASH
        );
        if GIT_HASH != "" {
            println!("GIT_HASH: {}", GIT_HASH);
        }
        if GIT_BRANCH != "" {
            println!("GIT_BRANCH: {}", GIT_BRANCH);
        }
        if BUILD_DATE != "" {
            println!("BUILD_DATE: {}", BUILD_DATE);
        }
        return;
    }

    let config_path = opt
        .config
        .unwrap_or_else(|| config::default_persistence_dir().join("conductor-config.toml"));
    let config_path_str = config_path.to_str().unwrap();

    println!("Using config path: {}", config_path_str);
    match bootstrap_from_config(config_path_str, Some(tracer)) {
        Ok(()) => {
            {
                let mut conductor_guard = CONDUCTOR.lock().unwrap();
                let conductor = conductor_guard.as_mut().expect("Conductor must be mounted");
                println!(
                    "Successfully loaded {} instance configurations",
                    conductor.instances().len()
                );
                println!("Starting instances...");
                conductor
                    .start_all_instances()
                    .expect("Could not start instances!");
                println!("Starting interfaces...");
                conductor.start_all_interfaces();
                // NB: the following println is very important!
                // Others are using it as an easy way to know that the interfaces have started.
                // Leave it as is!
                println!("{}", MAGIC_STRING);
                println!("Starting UI servers");
                conductor
                    .start_all_static_servers()
                    .expect("Could not start UI servers!");
            }

            match SignalConfiguration::default() {
                #[cfg(unix)]
                SignalConfiguration::Unix => {
                    let termination_signals =
                        Signals::new(&[SIGINT, SIGTERM]).expect("Couldn't create signals list");

                    // Wait forever until we get one of the signals defined above
                    let _sig = termination_signals.forever().next();

                    // So we're here because we received a shutdown signal.
                    // Let's shut down.
                    let conductor = {
                        let mut conductor_guard = CONDUCTOR.lock().unwrap();
                        std::mem::replace(&mut *conductor_guard, None)
                    };
                    let refs = Arc::strong_count(&CONDUCTOR);
                    if refs == 1 {
                        println!("Gracefully shutting down conductor...");
                    } else {
                        println!(
                                "Explicitly shutting down conductor. {} other threads were referencing it, so if unwrap errors follow, that might be why.",
                                refs - 1
                            );
                        conductor
                            .expect("No conductor running")
                            .shutdown()
                            .expect("Error shutting down conductor");
                    }
                    // NB: conductor is dropped here and should shut down itself
                }
                _ => (),
            }
        }
        Err(error) => println!("Error while trying to boot from config: {:?}", error),
    };
}

#[cfg_attr(tarpaulin, skip)]
fn bootstrap_from_config(path: &str, tracer: Option<ht::Tracer>) -> Result<(), HolochainError> {
    let config = load_config_file(&String::from(path))?;
    config
        .check_consistency(&mut Arc::new(Box::new(Conductor::load_dna)))
        .map_err(|string| HolochainError::ConfigError(string))?;
    mount_conductor_from_config(config, tracer);
    let mut conductor_guard = CONDUCTOR.lock().unwrap();
    let conductor = conductor_guard.as_mut().expect("Conductor must be mounted");
    println!("Unlocking agent keys:");
    conductor
        .config()
        .agents
        .iter()
        .map(|agent_config| {
            println!("Unlocking key for agent '{}': ", &agent_config.id);
            conductor.check_load_key_for_agent(&agent_config.id)
        })
        .collect::<Result<Vec<()>, String>>()
        .map_err(|error| HolochainError::ConfigError(error))?;
    conductor.boot_from_config()?;
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn load_config_file(path: &String) -> Result<Configuration, HolochainError> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(load_configuration::<Configuration>(&contents)?)
}
