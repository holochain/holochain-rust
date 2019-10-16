{ pkgs }:
let
  name = "hc-conductor-wasm-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-conductor-wasm-compile
  ( cd ./holochain_wasm/npm_package && npm install && npm test );
  '';
in
{
 buildInputs = [ script ];
}
