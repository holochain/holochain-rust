{ pkgs }:
let
  name = "hc-conductor-wasm-compile";

  script = pkgs.writeShellScriptBin name
  ''
  hc-conductor-wasm-install
  ( cd crates/holochain_wasm && cargo build --release -p holochain_conductor_wasm --target wasm32-unknown-unknown )
  wasm-bindgen --out-dir crates/holochain_wasm/npm_package/gen --nodejs target/wasm32-unknown-unknown/release/holochain_conductor_wasm.wasm
  '';
in
{
 buildInputs = [ script ];
}
