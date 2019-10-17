{ pkgs }:
let
  name = "hc-conductor-install";

  script = pkgs.writeShellScriptBin name
  ''
  cargo install -f --path crates/holochain
  '';
in
{
 buildInputs = [ script ];
}
