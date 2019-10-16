{ pkgs }:
let
  name = "hc-conductor-wasm-compile";

  script = pkgs.writeShellScriptBin name
  ''
  hc-conductor-wasm-install
  cd crates/holochain_wasm && cargo build --release -p holochain_conductor_wasm --target wasm32-unknown-unknown
  wasm-bindgen target/wasm32-unknown-unknown/release/holochain_conductor_wasm.wasm --out-dir npm_package/gen --nodejs
  '';
in
{
 buildInputs = [ script ];
}
