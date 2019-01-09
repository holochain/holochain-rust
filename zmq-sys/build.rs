extern crate metadeps;

use std::env;
use std::fs;
use std::path::Path;

fn prefix_dir(dir: &str) -> Option<String> {
    env::var("CARGO_MANIFEST_DIR").ok()
        .map(|prefix| Path::new(&prefix).join("vendor").join("zmq").join(dir))
        .and_then(|path| path.to_str().map(|p| p.to_owned()))
}

fn main() {
    // println!("{}", env::var("CARGO").unwrap());
    // println!("{}", env::var("OUT_DIR").unwrap());
    let dll_name = "libzmq-v140-mt-4_2_0.dll";
    #[cfg(windows)]
    fs::copy(
        Path::new(&prefix_dir("bin").unwrap()).join(dll_name),
        Path::new(&env::var("CARGO").unwrap()).parent().unwrap().join(dll_name),
    ).unwrap();
    println!("cargo:rustc-link-search=native={}", &prefix_dir("lib").unwrap());
    println!("cargo:include={}", &prefix_dir("include").unwrap());
    // panic!();
}
