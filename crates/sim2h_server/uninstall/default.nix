{ pkgs }:
let
  name = "hc-sim2h-server-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping sim2h-server binary from cargo home directory"
   rm -f $CARGO_HOME/bin/sim2h_server
  '';
in
{
 buildInputs = [ script ];
}
