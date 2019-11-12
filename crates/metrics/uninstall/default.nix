{ pkgs }:
let
  name = "hc-metrics-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping metrics binary from cargo home directory"
   rm -f $CARGO_HOME/bin/holochain_metrics
  '';
in
{
 buildInputs = [ script ];
}
