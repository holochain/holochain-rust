use std::{env, path::Path, process::Command};
/// Detect details about the Git repo to the cargo:rustc-env
/// - Set GIT_HASH to the current commit, eg. "3f9f2f5e07693c739639e1e84f8cdd483c52ef14", or all zeros if none
/// - Use HDK_VERSION, the the nearest upstream Git "tag", eg. "v0.0.32-alpha2-3-g3f9f2f5e0", or CARGO_PKG_VERSION
/// 
/// 
/// To allow running from somewhere *other* than an actual git checkout, we'll first check if these
/// environment variables are already set.  Then, fall back to git, and then fall back to all zeros
/// for GIT_HASH, and CARGO_PKG_VERSION for HDK_VERSION.
/// 
/// TODO: Decide whether we really want git involved; this prevents deterministic builds, eg. Nix packaging?
fn main() {
    if Path::new("../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../.git/HEAD");
    }

    let git_hash: String = match env::var_os("GIT_HASH") {
        Some(osstr) => osstr.into_string().expect("Unable to interpret GIT_HASH environment variable"),
        None => {
            let output = Command::new("git")
                .args(&["rev-parse", "HEAD"])
                .output()
                .expect("unable to execute git command");
            if output.status.success() {
                String::from_utf8(output.stdout).expect("Could not get GIT_HASH string from git rev-parse HEAD")
            } else {
                // Failed to find GIT_HASH or run git (or not in a repo); default to unknown
                "0000000000000000000000000000000000000000".to_owned()
            }
        }
    };
    println!("cargo:rustc-env=GIT_HASH={}", &git_hash);

    let git_describe: String = match env::var_os("HDK_VERSION") {
        Some(osstr) => osstr.into_string().expect("Unable to interpret HDK_VERSION environment variable"),
        None => {
            let output = Command::new("git")
                .args(&["describe"])
                .output()
                .expect("unable to execute git command");
            if output.status.success() {
                String::from_utf8(output.stdout).expect("Could not get HDK_VERSION string from git describe")
            } else {
                match env::var_os("CARGO_PKG_VERSION") {
                    Some(osstr) =>  osstr.into_string().expect("Unable to interpret CARGO_PKG_VERSION environment variable"),
                    None => panic!(
                        "git describe failed; set HDK_VERSION, or run build in holochain-rust Git repo core_types dir, not {:?}: {}",
                        env::current_dir(),
                        String::from_utf8(output.stderr).unwrap_or("<no output>".to_owned())),
                }
            }
        }
    };
    println!("cargo:rustc-env=HDK_VERSION={}", &git_describe);

    eprintln!(
        "core_types/build.rs: Setting cargo:rustc-env=HDK_VERSION={}, GIT_HASH={}",
        &git_describe, &git_hash
    );
}
