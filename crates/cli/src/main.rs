#![warn(unused_extern_crates)]
extern crate holochain_common;
extern crate holochain_conductor_lib;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_locksmith;
extern crate holochain_persistence_api;
extern crate holochain_persistence_file;
extern crate json_patch;
extern crate lib3h_sodium;
extern crate structopt;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate colored;
extern crate semver;
#[macro_use]
extern crate serde_json;
extern crate flate2;
extern crate glob;
extern crate ignore;
extern crate rpassword;
extern crate tar;
extern crate tempfile;
extern crate tera;

mod cli;
mod config_files;
mod error;
mod util;

use crate::error::{HolochainError, HolochainResult};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(StructOpt)]
/// A command line for Holochain
enum Cli {
    #[structopt(alias = "p")]
    ///  Builds DNA source files into a single .dna.json DNA file
    Package {
        #[structopt(long, short, parse(from_os_str))]
        output: Option<PathBuf>,
        #[structopt(long, short)]
        properties: Option<String>,
    },
    #[structopt(alias = "i")]
    /// Initializes a new Holochain app at the given directory
    Init {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    #[structopt(alias = "g")]
    /// Generates a new zome from a template
    Generate {
        #[structopt(parse(from_os_str))]
        /// The path to the zome that should be generated (usually in ./zomes/)
        zome: PathBuf,
        #[structopt(default_value = "rust")]
        /// Either the name of a built-in template (rust, rust-proc) or the url to a git repo containing a Zome template.
        template: String,
    },
    #[structopt(alias = "r")]
    /// Starts a development conductor with a websocket or http interface
    Run {
        #[structopt(long, short, default_value = "8888")]
        /// The port to run the websocket server at
        port: u16,
        #[structopt(long, short = "b")]
        /// Automatically package project before running
        package: bool,
        #[structopt(long = "dna", short = "d", parse(from_os_str))]
        /// Absolute path to the .dna.json file to run. [default: ./dist/<dna-name>.dna.json]
        dna_path: Option<PathBuf>,
        #[structopt(long)]
        /// Produce logging output
        logging: bool,
        #[structopt(long)]
        /// Save generated data to file system
        persist: bool,
        #[structopt(long)]
        /// Use real networking
        networked: bool,
        #[structopt(long, short, default_value = "websocket")]
        /// Specify interface type to use: websocket/http
        interface: String,
    },
    #[structopt(alias = "t")]
    /// Runs tests written in the test folder
    Test {
        #[structopt(long, short, default_value = "test")]
        /// The folder containing the test files
        dir: String,
        #[structopt(long, short, default_value = "test/index.js")]
        /// The path of the file to test
        testfile: String,
        #[structopt(long = "skip-package", short = "s")]
        /// Skip packaging DNA
        skip_build: bool,
        #[structopt(long = "show-npm-output", short = "n")]
        /// Show NPM output when installing test dependencies
        show_npm_output: bool,
    },
    #[structopt(name = "keygen", alias = "k")]
    /// Creates a new agent key pair, asks for a passphrase and writes an encrypted key bundle to disk in the XDG compliant config directory of Holochain, which is dependent on the OS platform (/home/alice/.config/holochain/keys or C:\\Users\\Alice\\AppData\\Roaming\\holochain\\holochain\\keys or /Users/Alice/Library/Preferences/com.holochain.holochain/keys)
    KeyGen {
        #[structopt(long, short, parse(from_os_str))]
        /// Specify path of file
        path: Option<PathBuf>,
        #[structopt(long, short)]
        /// Only print machine-readable output; intended for use by programs and scripts
        quiet: bool,
        #[structopt(long, short)]
        /// Don't ask for passphrase
        nullpass: bool,
    },
    #[structopt(name = "chain")]
    /// View the contents of a source chain
    ChainLog {
        #[structopt(name = "INSTANCE")]
        /// Instance ID to view
        instance_id: Option<String>,
        #[structopt(long, short, parse(from_os_str))]
        /// Location of chain storage
        path: Option<PathBuf>,
        #[structopt(long, short)]
        /// List available instances
        list: bool,
    },
    #[structopt(name = "hash")]
    /// Parse and hash a DNA file to determine its unique network hash
    HashDna {
        #[structopt(long, short, parse(from_os_str))]
        /// Path to .dna.json file [default: dist/<dna-name>.dna.json]
        path: Option<PathBuf>,
        #[structopt(long, short = "x")]
        /// Property (in the form 'name=value') that gets set/overwritten before calculating hash
        property: Option<Vec<String>>,
    },
}

fn main() {
    lib3h_sodium::check_init();
    run().unwrap_or_else(|err| {
        eprintln!("{}", err);

        ::std::process::exit(1);
    });
}

fn run() -> HolochainResult<()> {
    let args = Cli::from_args();

    let project_path =
        std::env::current_dir().map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
    match args {
        // If using default path, we'll create if necessary; otherwise, target dir must exist
        Cli::Package {
            output,
            properties: properties_string,
        } => {
            let output = if let Some(output_inner) = output {
                output_inner
            } else {
                util::std_package_path(&project_path).map_err(HolochainError::Default)?
            };

            let properties = properties_string
                .map(|s| serde_json::Value::from_str(&s))
                .unwrap_or_else(|| Ok(json!({})));

            match properties {
                Ok(properties) => {
                    cli::package(output, properties).map_err(HolochainError::Default)?
                }
                Err(e) => {
                    return Err(HolochainError::Default(format_err!(
                        "Failed to parse properties argument as JSON: {:?}",
                        e
                    )))
                }
            }
        }

        Cli::Init { path } => cli::init(&path).map_err(HolochainError::Default)?,

        Cli::Generate { zome, template } => {
            cli::generate(&zome, &template).map_err(HolochainError::Default)?
        }

        Cli::Run {
            package,
            port,
            dna_path,
            persist,
            networked,
            interface,
            logging,
        } => {
            let dna_path = dna_path
                .unwrap_or(util::std_package_path(&project_path).map_err(HolochainError::Default)?);
            let interface_type = cli::get_interface_type_string(interface);
            let conductor_config = cli::hc_run_configuration(
                &dna_path,
                port,
                persist,
                networked,
                &interface_type,
                logging,
            )
            .map_err(HolochainError::Default)?;
            cli::run(dna_path, package, port, interface_type, conductor_config)
                .map_err(HolochainError::Default)?
        }

        Cli::Test {
            dir,
            testfile,
            skip_build,
            show_npm_output,
        } => {
            let current_path = std::env::current_dir()
                .map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
            cli::test(&current_path, &dir, &testfile, skip_build, show_npm_output)
        }
        .map_err(HolochainError::Default)?,

        Cli::KeyGen {
            path,
            quiet,
            nullpass,
        } => {
            let passphrase = if nullpass {
                Some(String::from(holochain_common::DEFAULT_PASSPHRASE))
            } else {
                None
            };
            cli::keygen(path, passphrase, quiet)
                .map_err(|e| HolochainError::Default(format_err!("{}", e)))?
        }

        Cli::ChainLog {
            instance_id,
            list,
            path,
        } => match (list, instance_id) {
            (true, _) => cli::chain_list(path),
            (false, None) => {
                Cli::clap().print_help().expect("Couldn't print help!");
                println!("\n\nTry `hc help chain` for more info");
            }
            (false, Some(instance_id)) => {
                cli::chain_log(path, instance_id)
                    .map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
            }
        },
        Cli::HashDna { path, property } => {
            let dna_path = path
                .unwrap_or(util::std_package_path(&project_path).map_err(HolochainError::Default)?);

            let dna_hash = cli::hash_dna(&dna_path, property)
                .map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
            println!("DNA Hash: {}", dna_hash);
        }
    }

    Ok(())
}
