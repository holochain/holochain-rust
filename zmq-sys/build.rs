extern crate metadeps;

use std::env;
#[cfg(windows)]
use std::fs;
use std::path::Path;

fn prefix_dir(dir: &str) -> Option<String> {
    env::var("CARGO_MANIFEST_DIR").ok()
        .map(|prefix| Path::new(&prefix).join("vendor").join("zmq").join(dir))
        .and_then(|path| path.to_str().map(|p| p.to_owned()))
}

fn main() {

    #[cfg(windows)]
    // hack to get the libzmq dll on the PATH
    // copies it next to cargo
    // kind of messy, but it is less than 1MB
    {
        let dll_name = "libzmq-v140-mt-4_2_0.dll";
        let cargo_path = &env::var("CARGO").unwrap();
        let path_on_path = Path::new(cargo_path).parent().unwrap().join("libzmq");
        fs::create_dir_all(&path_on_path).unwrap();
        fs::copy(
            Path::new(&prefix_dir("bin").unwrap()).join(dll_name),
            path_on_path.join(dll_name),
        ).unwrap();
    }

    println!("cargo:rustc-link-search=native={}", &prefix_dir("lib").unwrap());
    println!("cargo:include={}", &prefix_dir("include").unwrap());
}
