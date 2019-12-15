{ pkgs }:
let
  name = "hc-conductor-wasm-compile";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  hc-conductor-wasm-bindgen-install
  echo $CARGO_TARGET_DIR
  ( cd crates/holochain_wasm && cargo build --release -p holochain_conductor_wasm --target wasm32-unknown-unknown )
  wasm-bindgen --out-dir ./crates/holochain_wasm/npm_package/gen --nodejs "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/holochain_conductor_wasm.wasm
  '';
in
{
 buildInputs = [ script ];
}
