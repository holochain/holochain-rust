{ pkgs }:
let
 name = "hc-rust-wasm-compile";

 paths = [
  "crates/hdk/wasm-test"
  "crates/conductor_lib/wasm-test"
  "crates/conductor_lib/test-bridge-caller"
  "crates/core/src/nucleus/actions/wasm-test"
 ];

 compile = path:
 ''
 set -euxo pipefail
 export WASM_PATH=${path}/
 cargo build --release --target wasm32-unknown-unknown --manifest-path "$WASM_PATH"Cargo.toml --target-dir "''${HC_TARGET_PREFIX:-.}"/"$WASM_PATH"target;
 '';

 script = pkgs.writeShellScriptBin name
 ''
 ${pkgs.lib.concatMapStrings (path: compile path) paths}
 '';
in
{
 buildInputs = [ script ];
}
