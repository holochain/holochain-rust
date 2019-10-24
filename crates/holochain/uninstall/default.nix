{ pkgs }:
let
  name = "hc-conductor-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping holochain conductor binary from cargo home directory"
   rm -f $CARGO_HOME/bin/holochain
  '';
in
{
 buildInputs = [ script ];
}
