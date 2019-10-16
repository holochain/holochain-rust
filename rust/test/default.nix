{ holonix, pkgs }:
let
  name = "hc-rust-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-rust-wasm-compile;
  cd crates/cli && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../conductor_api && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../conductor_lib && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../core && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../core_types && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../dpki && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../hdk && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../hdk-v2 && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../holochain && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../holochain_wasm && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../net && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  cd ../wasm_utils && HC_SIMPLE_LOGGER_MUTE=1 RUST_BACKTRACE=1 cargo test --all --target-dir "$HC_TARGET_PREFIX"target "$1" -- --test-threads=${holonix.rust.test.threads};
  '';
in
{
 buildInputs = [ script ];
}
