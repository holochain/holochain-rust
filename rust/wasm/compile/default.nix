{ pkgs }:
let
 name = "hc-rust-wasm-compile";

 paths = [
  "hdk-rust/wasm-test"
  "wasm_utils/wasm-test/integration-test"
  "conductor_api/wasm-test"
  "conductor_api/test-bridge-caller"
  "core/src/nucleus/actions/wasm-test"
 ];

 compile = path:
 ''
 export WASM_PATH=${path}/
 cargo build --release --target wasm32-unknown-unknown --manifest-path "$WASM_PATH"Cargo.toml --target-dir "$HC_TARGET_PREFIX""$WASM_PATH"target;
 '';

 script = pkgs.writeShellScriptBin name
 ''
 ${pkgs.lib.concatMapStrings (path: compile path) paths}
 '';
in
{
 buildInputs = [ script ];
}
