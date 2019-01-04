extern crate holochain_cas_implementations;
extern crate holochain_container_api;
extern crate holochain_core;
extern crate holochain_core_types;
extern crate holochain_net;
extern crate holochain_wasm_utils;
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate assert_cmd;
extern crate base64;
extern crate colored;
extern crate dir_diff;
extern crate semver;
extern crate toml;
#[macro_use]
extern crate serde_json;
extern crate ignore;
extern crate rustyline;
extern crate tempfile;
extern crate uuid;

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
        name = "agent",
        alias = "a",
        about = "Starts a Holochain node as an agent"
    )]
    Agent,
    #[structopt(
        name = "package",
        alias = "p",
        about = "Builds the current Holochain app into a .hcpkg file"
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
        about = "Generates a new zome and scaffolds the given capabilities"
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
        about = "Starts a development container with an open websocket"
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
}

fn main() {
    run().unwrap_or_else(|err| {
        eprintln!("{}", err);

        ::std::process::exit(1);
    });
}

fn run() -> HolochainResult<()> {
    let args = Cli::from_args();

    match args {
        Cli::Agent => cli::agent().map_err(HolochainError::Default)?,
        Cli::Package { strip_meta, output } => {
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
        } => cli::run(package, port, persist).map_err(HolochainError::Default)?,
        Cli::Test {
            dir,
            testfile,
            skip_build,
        } => cli::test(std::env::current_dir()?, &dir, &testfile, skip_build)
            .map_err(HolochainError::Default)?,
    }

    Ok(())
}
