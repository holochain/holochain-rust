use std::{path::Path, process::Command};
fn main() {
    if Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("unable to execute git command");

    let git_hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let output = Command::new("git")
        .args(&["describe"])
        .output()
        .expect("unable to execute git command");

    println!(
        "cargo:rustc-env=HDK_VERSION={}",
        String::from_utf8(output.stdout).expect("Could not get string from git output")
    )
}
