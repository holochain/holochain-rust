use std::env;
/// Detect details about the HDK Version being built, to make available as hdk::HDK_VERSION variable
/// - Use supplied "HDK_VERSION" or "CARGO_PKG_VERSION" environment variables
///   - Should match the nearest upstream Git "tag", eg. "v0.0.32-alpha2-3-g3f9f2f5e0", but
///     since the source code may *not* be an actual Git repo, we can't really check this.
///
/// - Use the Nix build target directory path to find the HDK_HASH; we'll look at a
///   supplied "HDK_HASH" variable, or break down the Nix-supplied "out" variable directly:
///
///     $ declare -x out="/nix/store/w7vyf4x77b1539rxakcqni8zdidpg7gy-some-target"
///     $ basename $out | cut -d- -f1
///     w7vyf4x77b1539rxakcqni8zdidpg7gy
///
/// Using these assumptions, we can build using raw cargo build (without a Nix-supplied environment),
/// and we'll get an unknown HDK_HASH="000...", and a valid HDK_VERSION="0.0.32-alpha2"; but, you can
/// supply a different HDK_HASH, if you wish.   For a Nix build, we'll obtain the HDK_HASH from the
/// "out" environment variable.
///
fn main() {
    let hdk_hash: String = env::var("HDK_HASH")
        .ok()
        .or_else(|| {
            env::var("out").ok().and_then(|out| {
                out.split('/').last().and_then(|basename| {
                    basename
                        .split('-')
                        .nth(0)
                        .and_then(|hash| Some(hash.to_string()))
                })
            })
        })
        .unwrap_or_else(|| "00000000000000000000000000000000".to_string());
    println!("cargo:rustc-env=HDK_HASH={}", hdk_hash);

    let hdk_version: String = env::var("HDK_VERSION")
        .or_else(|_| env::var("CARGO_PKG_VERSION"))
        .expect("Cannot deduce HDK_VERSION; ensure HDK_VERSION or CARGO_PKG_VERSION (via Cargo.toml [package] version) is set");
    assert!(
        hdk_version.len() > 0,
        "Invalid HDK_VERSION: {:?}",
        &hdk_version
    );
    println!("cargo:rustc-env=HDK_VERSION={}", &hdk_version);
}
