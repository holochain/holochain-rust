{ pkgs }:
let
  script-stress-sim1h = pkgs.writeShellScriptBin "hc-stress-test-sim1h"
  ''
  hc-cli-install
  hc-conductor-install
   ( cd stress-test && npm install && APP_SPEC_NETWORK_TYPE=sim1h AWS_ACCESS_KEY_ID=bla AWS_SECRET_ACCESS_KEY=blup npm test -- "$@" )
  '';

  script-stress-sim2h = pkgs.writeShellScriptBin "hc-stress-test-sim2h"
  ''
  hc-cli-install
  hc-conductor-install
   ( cd stress-test && npm install && APP_SPEC_NETWORK_TYPE=sim2h npm test -- "$@" )
  '';
in
{
 buildInputs = [ script-stress-sim1h script-stress-sim2h ];
}
