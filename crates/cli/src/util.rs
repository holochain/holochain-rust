use crate::error::DefaultResult;
use colored::*;
pub use holochain_common::paths::DNA_EXTENSION;
use std::{
    fs,
    io::ErrorKind,
    path::PathBuf,
    process::{Command, Stdio},
};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
pub fn run_cmd(base_path: &PathBuf, bin: String, args: &[&str]) -> DefaultResult<()> {
    let pretty_command = format!("{} {}", bin.green(), args.join(" ").cyan());

    println!("> {}", pretty_command);

    let command_string = format!("{} {}", bin, args.join(" "));
    println!("{:?}", command_string);

    // this bypasses the native rust handling of Command by passing the whole thing to bash as:
    // bash -c "{{ thing you wanted to do }}"
    // it's basically `eval` which is evil :(
    // we do this because Rust's `Command` is designed to be "portable"
    // i.e. the same binary should work across bash, powershell, CMD, etc.
    // to achieve this all the arg strings are built internally by the rust binary then run as-is
    // so e.g. Command::new("echo").args("$PWD") would literally print `"$PWD"` not the contents of
    // the $PWD environment variable (e.g. because Windows CMD would expect `%cd%` not `$PWD`)
    // what we want is for things to be "portable"
    // i.e. developers can configure their machines with environment variables and have `hc`
    // evaluate them to local values that fit their personal workflow/configurations
    // so e.g. the following should work (i.e. `wasm-gc` finds the built .wasm file)
    // {
    //  "command": "cargo",
    //  "arguments": [
    //   "build",
    //   "--release",
    //   "--target=wasm32-unknown-unknown"
    //  ]
    // },
    // {
    //  "command": "wasm-gc",
    //  "arguments": ["$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/{{ name }}.wasm"]
    // },
    // note the implicit (in cargo) and explicit (in wasm-gc) use of $CARGO_TARGET_DIR!
    // note also that the supported development environment is based on `nix-shell` so we _already_
    // assume/expect that developers are using something bash-like for development already
    // e.g. we assume `cargo`, `wasm-gc`, `wasm-opt`, `wasm2wat`, `wat2wasm` all exist in the
    // default template (which we can't assume outside nix-shell in a portable way).
    //
    // Note the use of --norc. This is to prevent the shell from loading up its usual startup
    // script, which may echo things to STDOUT. The reason this matters is that some of the
    // commands in a .hcbuild file (which are fed to this function) capture from STDOUT and do
    // something meaningful with the value. Do we really want a MOTD in our build artifact path?
    //
    // @TODO - does it make more sense to push "execute arbitrary bash" style features down to the
    // `nix-shell` layer where we have a better toolkit to handle environments/dependencies?
    // e.g. @see `hn-release-cut` from holonix that implements conventions/hooks to standardise
    // bash processes in an extensible way
    let status = Command::new("bash")
        .args(&["--norc", "-c", &command_string])
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
pub fn std_dna_file_name(path: &PathBuf) -> DefaultResult<String> {
    let dir_name = file_name_string(path)?;
    Ok(format!("{}.{}", dir_name, DNA_EXTENSION))
}

pub const DIST_DIR_NAME: &str = "dist";

/// Helper method for obtaining the path to the dist directory, and creating it if it doesn't exist
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
pub fn get_dist_path(path: &PathBuf) -> DefaultResult<PathBuf> {
    // create dist folder
    let dist_path = path.join(&DIST_DIR_NAME);

    if !dist_path.exists() {
        fs::create_dir(dist_path.as_path())
            .map_err(|e| format_err!("Couldn't create path {:?}; {}", dist_path, e))?;
    }
    Ok(dist_path)
}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
pub fn std_package_path(path: &PathBuf) -> DefaultResult<PathBuf> {
    Ok(get_dist_path(path)?.join(std_dna_file_name(path)?))
}

/// Helper method for obtaining the file name of a path as a String
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CLI)]
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
                    if let Some(messages) = extra_help {
                        for message in messages {
                            println!("{}", message)
                        }
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

#[cfg(test)]
pub mod tests {

    use cli::init::tests::gen_dir;
    use util::run_cmd;

    #[test]
    fn run_cmd_test() {
        let dir = gen_dir();
        let dir_path_buf = &dir.path().to_path_buf();

        // test this manually with:
        // `cargo test -p hc run_cmd_test -- --nocapture`
        // the tempdir should be echoed and not a literal '$PWD'
        assert!(run_cmd(dir_path_buf, "echo".to_string(), &["$PWD"]).is_ok());
    }
}
