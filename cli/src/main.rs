#![warn(unused_extern_crates)]
extern crate holochain_common;
extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_json_api;
extern crate holochain_persistence_api;
extern crate holochain_persistence_file;
extern crate holochain_wasm_utils;
extern crate lib3h_sodium;
extern crate structopt;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate colored;
extern crate semver;
extern crate toml;
#[macro_use]
extern crate serde_json;
extern crate ignore;
extern crate rpassword;

mod cli;
mod config_files;
mod error;
mod util;

use crate::error::{HolochainError, HolochainResult};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "A command line for Holochain")]
enum Cli {
    #[structopt(
        name = "package",
        alias = "p",
        about = "Builds DNA source files into a single .dna.json DNA file"
    )]
    Package {
        #[structopt(
            long = "strip-meta",
            help = "Strips all __META__ sections off the target bundle. Makes unpacking of the bundle impossible"
        )]
        strip_meta: bool,
        #[structopt(long = "output", short = "o", parse(from_os_str))]
        output: Option<PathBuf>,
        #[structopt(long = "properties", short = "p")]
        properties: Option<String>,
    },
    #[structopt(
        name = "unpack",
        about = "Unpacks a Holochain bundle into it's original file system structure"
    )]
    Unpack {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(parse(from_os_str))]
        to: PathBuf,
    },
    #[structopt(
        name = "init",
        alias = "i",
        about = "Initializes a new Holochain app at the given directory"
    )]
    Init {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    #[structopt(
        name = "generate",
        alias = "g",
        about = "Generates a new zome and scaffolds the given functions"
    )]
    Generate {
        #[structopt(
            help = "The path to the zome that should be generated (usually in ./zomes/)",
            parse(from_os_str)
        )]
        zome: PathBuf,
        #[structopt(help = "The language of the generated zome", default_value = "rust")]
        language: String,
    },
    #[structopt(
        name = "run",
        alias = "r",
        about = "Starts a development conductor with a websocket or http interface"
    )]
    Run {
        #[structopt(
            long,
            short,
            help = "The port to run the websocket server at",
            default_value = "8888"
        )]
        port: u16,
        #[structopt(
            long,
            short = "b",
            help = "Automatically package project before running"
        )]
        package: bool,
        #[structopt(
            long = "dna",
            short = "d",
            help = "Absolute path to the .dna.json file to run. [default: ./dist/<dna-name>.dna.json]"
        )]
        dna_path: Option<PathBuf>,
        #[structopt(long, help = "Produce logging output")]
        logging: bool,
        #[structopt(long, help = "Save generated data to file system")]
        persist: bool,
        #[structopt(long, help = "Use real networking")]
        networked: bool,
        #[structopt(
            long,
            short,
            help = "Specify interface type to use: websocket/http",
            default_value = "websocket"
        )]
        interface: String,
    },
    #[structopt(
        name = "test",
        alias = "t",
        about = "Runs tests written in the test folder"
    )]
    Test {
        #[structopt(
            long,
            short,
            default_value = "test",
            help = "The folder containing the test files"
        )]
        dir: String,
        #[structopt(
            long,
            short,
            default_value = "test/index.js",
            help = "The path of the file to test"
        )]
        testfile: String,
        #[structopt(long = "skip-package", short = "s", help = "Skip packaging DNA")]
        skip_build: bool,
        #[structopt(
            long = "show-npm-output",
            short = "n",
            help = "Show NPM output when installing test dependencies"
        )]
        show_npm_output: bool,
    },
    #[structopt(
        name = "keygen",
        alias = "k",
        about = "Creates a new agent key pair, asks for a passphrase and writes an encrypted key bundle to disk in the XDG compliant config directory of Holochain, which is dependent on the OS platform (/home/alice/.config/holochain/keys or C:\\Users\\Alice\\AppData\\Roaming\\holochain\\holochain\\keys or /Users/Alice/Library/Preferences/com.holochain.holochain/keys)"
    )]
    KeyGen {
        #[structopt(long, short, help = "Specify path of file")]
        path: Option<PathBuf>,
        #[structopt(
            long,
            short,
            help = "Only print machine-readable output; intended for use by programs and scripts"
        )]
        quiet: bool,
        #[structopt(long, short, help = "Don't ask for passphrase")]
        nullpass: bool,
    },
    #[structopt(name = "chain", about = "View the contents of a source chain")]
    ChainLog {
        #[structopt(name = "INSTANCE", help = "Instance ID to view")]
        instance_id: Option<String>,
        #[structopt(long, short, help = "Location of chain storage")]
        path: Option<PathBuf>,
        #[structopt(long, short, help = "List available instances")]
        list: bool,
    },
    #[structopt(
        name = "hash",
        about = "Parse and hash a DNA file to determine its unique network hash"
    )]
    HashDna {
        #[structopt(
            long,
            short,
            help = "Path to .dna.json file [default: dist/<dna-name>.dna.json]"
        )]
        path: Option<PathBuf>,
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
            strip_meta,
            output,
            properties: properties_string,
        } => {
            let output = if output.is_some() {
                output.unwrap()
            } else {
                util::std_package_path(&project_path).map_err(HolochainError::Default)?
            };

            let properties = properties_string
                .map(|s| serde_json::Value::from_str(&s))
                .unwrap_or_else(|| Ok(json!({})));

            match properties {
                Ok(properties) => {
                    cli::package(strip_meta, output, properties).map_err(HolochainError::Default)?
                }
                Err(e) => {
                    return Err(HolochainError::Default(format_err!(
                        "Failed to parse properties argument as JSON: {:?}",
                        e
                    )))
                }
            }
        }

        Cli::Unpack { path, to } => cli::unpack(&path, &to).map_err(HolochainError::Default)?,

        Cli::Init { path } => cli::init(&path).map_err(HolochainError::Default)?,

        Cli::Generate { zome, language } => {
            cli::generate(&zome, &language).map_err(HolochainError::Default)?
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
                dna_path.clone(),
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
        Cli::HashDna { path } => {
            let dna_path = path
                .unwrap_or(util::std_package_path(&project_path).map_err(HolochainError::Default)?);

            let dna_hash = cli::hash_dna(&dna_path)
                .map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
            println!("DNA Hash: {}", dna_hash);
        }
    }

    Ok(())
}
