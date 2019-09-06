use crate::error::DefaultResult;
use colored::*;
pub use holochain_common::paths::DNA_EXTENSION;
use holochain_core_types::error::HcResult;
use holochain_dpki::seed::{EncryptedSeed, MnemonicableSeed, Seed, SeedType, TypedSeed};
use rpassword;
use std::io::stdin;

use std::{
    fs,
    io::{self, ErrorKind, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn get_secure_string_double_check(name: &str, quiet: bool) -> HcResult<String> {
    if !quiet {
        print!("Enter {}: ", name);
        io::stdout().flush()?;
    }
    let retrieved_str_1 = rpassword::read_password()?;
    if !quiet {
        print!("Re-enter {}: ", name);
        io::stdout().flush()?;
    }
    let retrieved_str_2 = rpassword::read_password()?;
    if retrieved_str_1 != retrieved_str_2 {
        panic!("Root seeds do not match. Aborting");
    }
    Ok(retrieved_str_1)
}

pub fn user_prompt(message: &str, quiet: bool) {
    if !quiet {
        println!("{}", message);
    }
}

pub fn user_prompt_yes_no(message: &str, quiet: bool) -> bool {
    user_prompt(format!("{} (Y/n)", message).as_str(), quiet);
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Could not read from stdin");
    match input.as_str() {
        "Y\n" => true,
        "n\n" => false,
        _ => panic!(format!("Invalid response: {}", input)),
    }
}

/// Retrieve a seed from a BIP39 mnemonic
/// If a passphrase is provided assume it is encrypted and decrypt it
/// If not then assume it is unencrypted
pub fn get_seed(
    seed_mnemonic: String,
    passphrase: Option<String>,
    seed_type: SeedType,
) -> HcResult<TypedSeed> {
    match passphrase {
        Some(passphrase) => {
            EncryptedSeed::new_with_mnemonic(seed_mnemonic, seed_type)?.decrypt(passphrase, None)
        }
        None => Seed::new_with_mnemonic(seed_mnemonic, seed_type)?.into_typed(),
    }
}

pub trait WordCountable {
    fn word_count(&self) -> usize;
}

impl WordCountable for String {
    fn word_count(&self) -> usize {
        self.split(" ").count()
    }
}

pub fn run_cmd(base_path: PathBuf, bin: String, args: &[&str]) -> DefaultResult<()> {
    let pretty_command = format!("{} {}", bin.green(), args.join(" ").cyan());

    println!("> {}", pretty_command);

    let status = Command::new(bin)
        .args(args)
        .current_dir(base_path)
        .status()?;

    ensure!(
        status.success(),
        "command {} was not successful",
        pretty_command
    );

    Ok(())
}

/// Helper method for getting the standard dna file name built from the directory name and extension
pub fn std_dna_file_name(path: &PathBuf) -> DefaultResult<String> {
    let dir_name = file_name_string(path)?;
    Ok(format!("{}.{}", dir_name, DNA_EXTENSION))
}

pub const DIST_DIR_NAME: &str = "dist";

/// Helper method for obtaining the path to the dist directory, and creating it if it doesn't exist
pub fn get_dist_path(path: &PathBuf) -> DefaultResult<PathBuf> {
    // create dist folder
    let dist_path = path.join(&DIST_DIR_NAME);

    if !dist_path.exists() {
        fs::create_dir(dist_path.as_path())
            .map_err(|e| format_err!("Couldn't create path {:?}; {}", dist_path, e))?;
    }
    Ok(dist_path)
}

pub fn std_package_path(path: &PathBuf) -> DefaultResult<PathBuf> {
    Ok(get_dist_path(path)?.join(std_dna_file_name(path)?))
}

/// Helper method for obtaining the file name of a path as a String
pub fn file_name_string(path: &PathBuf) -> DefaultResult<String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| format_err!("unable to retrieve file name for path: {:?}", path))?
        .to_str()
        .ok_or_else(|| format_err!("unable to convert file name to string"))?;

    Ok(file_name.into())
}

/// Helper method for CLI commands that require cargo to be installed
/// Takes in extra contextual info as strings, and returns a bool
/// which should indicate whether the caller should continue with execution
/// or perform a graceful and early exit
pub fn check_for_cargo(use_case: &str, extra_help: Option<Vec<&str>>) -> DefaultResult<bool> {
    match Command::new("cargo").stdout(Stdio::null()).status() {
        // no problems checking, and cargo is installed
        Ok(_) => Ok(true),
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    println!("This command requires the `cargo` command, which is part of the Rust toolchain.");
                    println!("{}", use_case);
                    println!("It is important that you use the correct version of `cargo`.");
                    println!("We recommend you work inside a nix-shell or use nix-env to install `hc`.");
                    println!("For more information see https://docs.holochain.love");
                    match extra_help {
                        Some(messages) => {
                            for message in messages {
                                println!("{}", message)
                            }
                        },
                        None => {}
                    };
                    println!("Having taken those steps, retry this command.");
                    // early exit with Ok, but false (meaning don't continue) since this is the graceful exit
                    Ok(false)
                }
                // convert from a std::io::Error into a failure::Error
                // and actually return that error since it's something
                // different than just not finding `cargo`
                _ => Err(format_err!("This command requires the `cargo` command, but there was an error checking if it is available or not: {}", e)),
            }
        }
    }
}
