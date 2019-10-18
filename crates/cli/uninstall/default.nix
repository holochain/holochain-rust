{ pkgs }:
let
  name = "hc-cli-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping hc binary from cargo home directory"
   rm -f $CARGO_HOME/bin/hc
  '';
in
{
 buildInputs = [ script ];
}
