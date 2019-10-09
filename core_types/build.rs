use std::{env, path::Path, process::Command};
/// Detect details about the Git repo to the cargo:rustc-env
/// - Set GIT_HASH to the current commit, eg. "3f9f2f5e07693c739639e1e84f8cdd483c52ef14"
/// - Set HDK_VERSION to the nearest upstream Git "tag", eg. "v0.0.32-alpha2-3-g3f9f2f5e0"
/// 
/// To allow running from somewhere *other* than an actual git checkout, we'll first check if these
/// environment variables are already set.
/// 
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
            assert!(output.status.success(),
                    "git rev-parse HEAD failed; Set GIT_HASH, or run build in holochain-rust Git repo core_types dir, not {:?}: {}",
                    env::current_dir(),
                    String::from_utf8(output.stderr).unwrap_or("<no output>".to_owned()));
            String::from_utf8(output.stdout).expect("Could not get GIT_HASH string from git rev-parse HEAD")
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
            assert!(output.status.success(),
                    "git describe failed; set HDK_VERSION, or run build in holochain-rust Git repo core_types dir, not {:?}: {}",
                    env::current_dir(),
                    String::from_utf8(output.stderr).unwrap_or("<no output>".to_owned()));
            String::from_utf8(output.stdout).expect("Could not get HDK_VERSION string from git describe")
        }
    };
    println!("cargo:rustc-env=HDK_VERSION={}", &git_describe);

    eprintln!(
        "core_types/build.rs: Setting cargo:rustc-env=HDK_VERSION={}, GIT_HASH={}",
        &git_describe, &git_hash
    );
}
