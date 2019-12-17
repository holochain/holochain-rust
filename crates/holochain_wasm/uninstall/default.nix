{ pkgs }:
let
  name = "hc-conductor-wasm-bindgen-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
  cargo uninstall wasm-bindgen-cli
  '';
in
{
 buildInputs = [ script ];
}
