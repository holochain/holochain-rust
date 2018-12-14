use crate::error::DefaultResult;
use colored::*;
use std::{path::PathBuf, process::Command};

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
