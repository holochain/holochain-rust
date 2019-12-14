{ pkgs }:
let
  name = "hc-conductor-wasm-compile";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  hc-conductor-wasm-bindgen-install
  ( cd crates/holochain_wasm && cargo build --target-dir "$HC_TARGET_PREFIX"/target --release -p holochain_conductor_wasm --target wasm32-unknown-unknown )
  wasm-bindgen --out-dir "$HC_TARGET_PREFIX"crates/holochain_wasm/npm_package/gen --nodejs "$HC_TARGET_PREFIX"/target/wasm32-unknown-unknown/release/holochain_conductor_wasm.wasm
  '';
in
{
 buildInputs = [ script ];
}
