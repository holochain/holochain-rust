use std::path::PathBuf;

pub fn wasm_target_dir(test_path: &PathBuf, wasm_path: &PathBuf) -> PathBuf {
    // this env var checker can't use holochain_common
    // crate because that uses `directories` crate which doesn't compile to WASM
    let mut target_dir = PathBuf::new();
    if let Ok(prefix) = std::env::var("HC_TARGET_PREFIX") {
        target_dir.push(PathBuf::from(prefix));
        target_dir.push("crates");
        target_dir.push(test_path);
    }
    target_dir.push(wasm_path);
    target_dir.push(PathBuf::from("target"));

    target_dir
}
