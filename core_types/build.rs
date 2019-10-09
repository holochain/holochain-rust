use std::{path::Path, process::Command};
fn main() {
    if Path::new("../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../.git/HEAD");
    }
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("unable to execute git command");

    let git_hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_HASH={}", &git_hash);

    let output = Command::new("git")
        .args(&["describe"])
        .output()
        .expect("unable to execute git command");
    
    let describe = String::from_utf8(output.stdout).expect("Could not get string from git output");
    println!(
        "cargo:rustc-env=HDK_VERSION={}",
        &describe
    );
    eprintln!(
        "core_types/build.rs: Setting cargo:rustc-env=HDK_VERSION={}, GIT_HASH={}",
        &describe, &git_hash
    );
}
