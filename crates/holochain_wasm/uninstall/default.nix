{ pkgs }:
let
  name = "hc-conductor-wasm-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
  cargo uninstall wasm-bindgen-cli
  '';
in
{
 buildInputs = [ script ];
}
