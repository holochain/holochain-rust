use std::env;
/// Detect details about the HDK Version being built, to make available as hdk::HDK_VERSION variable
/// - Use HDK_VERSION or CARGO_PKG_VERSION environment variables
///   - Should match the nearest upstream Git "tag", eg. "v0.0.32-alpha2-3-g3f9f2f5e0", but
///     since the source code may *not* be an actual Git repo, we can't really check this.
fn main() {
    let hdk_version: String = env::var("HDK_VERSION")
        .or_else( |_| env::var("CARGO_PKG_VERSION"))
        .expect("Cannot deduce HDK_VERSION; ensure HDK_VERSION or CARGO_PKG_VERSION (via Cargo.toml [package] version) is set");
    assert!( hdk_version.len() > 0,
             "Invalid HDK_VERSION: {:?}", &hdk_version );
    println!("cargo:rustc-env=HDK_VERSION={}", &hdk_version);
}
