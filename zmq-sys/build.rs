extern crate metadeps;

use std::env;
use std::path::Path;

fn prefix_dir(dir: &str) -> Option<String> {
    env::var("CARGO_MANIFEST_DIR").ok()
        .map(|prefix| Path::new(&prefix).join("vendor").join("zmq").join(dir))
        .and_then(|path| path.to_str().map(|p| p.to_owned()))
}

fn main() {
    println!("cargo:rustc-link-search=native={}", &prefix_dir("lib").unwrap());
    println!("cargo:include={}", &prefix_dir("include").unwrap());
}
