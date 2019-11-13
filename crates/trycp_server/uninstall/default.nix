{ pkgs }:
let
  name = "hc-trycp-server-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping trycp-server binary from cargo home directory"
   rm -f $CARGO_HOME/bin/trycp_server
  '';
in
{
 buildInputs = [ script ];
}
