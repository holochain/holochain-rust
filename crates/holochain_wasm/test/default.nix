{ pkgs }:
let
  name = "hc-conductor-wasm-test";

  script = pkgs.writeShellScriptBin name
  ''
  set -euxo pipefail
  hc-conductor-wasm-compile
  ( cd crates/holochain_wasm/npm_package && npm install && npm test );
  '';
in
{
 buildInputs = [ script ];
}
