#![warn(unused_extern_crates)]
extern crate holochain_common;
extern crate holochain_conductor_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_dpki;
extern crate holochain_sodium;
extern crate holochain_wasm_utils;
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
use std::path::PathBuf;
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
    },
    #[structopt(
        name = "keygen",
        alias = "k",
        about = "Creates a new agent key pair, asks for a passphrase and writes an encrypted key bundle to disk in the XDG compliant config directory of Holochain, which is dependent on the OS platform (/home/alice/.config/holochain/keys or C:\\Users\\Alice\\AppData\\Roaming\\holochain\\holochain\\keys or /Users/Alice/Library/Preferences/com.holochain.holochain/keys)"
    )]
    KeyGen,
}

fn main() {
    holochain_sodium::check_init();
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
        Cli::Package { strip_meta, output } => {
            let output = if output.is_some() {
                output.unwrap()
            } else {
                util::std_package_path(&project_path).map_err(HolochainError::Default)?
            };
            cli::package(strip_meta, output).map_err(HolochainError::Default)?
        }
        Cli::Unpack { path, to } => cli::unpack(&path, &to).map_err(HolochainError::Default)?,
        Cli::Init { path } => cli::init(&path).map_err(HolochainError::Default)?,
        Cli::Generate { zome, language } => {
            cli::generate(&zome, &language).map_err(HolochainError::Default)?
        }
        Cli::Run {
            package,
            port,
            persist,
            networked,
            interface,
        } => {
            let dna_path =
                util::std_package_path(&project_path).map_err(HolochainError::Default)?;
            let interface_type = cli::get_interface_type_string(interface);
            let conductor_config =
                cli::hc_run_configuration(&dna_path, port, persist, networked, &interface_type)
                    .map_err(HolochainError::Default)?;
            cli::run(dna_path, package, port, interface_type, conductor_config)
                .map_err(HolochainError::Default)?
        }
        Cli::Test {
            dir,
            testfile,
            skip_build,
        } => {
            let current_path = std::env::current_dir()
                .map_err(|e| HolochainError::Default(format_err!("{}", e)))?;
            cli::test(&current_path, &dir, &testfile, skip_build)
        }
        .map_err(HolochainError::Default)?,
        Cli::KeyGen => {
            cli::keygen(None, None).map_err(|e| HolochainError::Default(format_err!("{}", e)))?
        }
    }

    Ok(())
}
