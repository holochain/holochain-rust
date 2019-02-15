use crate::error::DefaultResult;
use colored::*;
use std::{
    io::ErrorKind,
    path::PathBuf,
    process::{Command, Stdio},
};

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
                    println!("As a first step, get Rust installed by using rustup https://rustup.rs/.");
                    println!("Holochain requires you use the nightly-2019-01-24 toolchain.");
                    println!("With Rust already installed switch to it by running the following commands:");
                    println!("$ rustup toolchain install nightly-2019-01-24");
                    println!("$ rustup default nightly-2019-01-24");
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
