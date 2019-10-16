{ pkgs }:
let
  name-stress-sim1h = "hc-stress-test-sim1h";

  script-stress-sim1h = pkgs.writeShellScriptBin name-stress-sim1h
  ''
  hc-cli-install
  hc-conductor-install
   ( cd stress-test && npm install && AWS_ACCESS_KEY_ID=bla AWS_SECRET_ACCESS_KEY=blup npm test )
  '';

  name-stress-sim2h = "hc-stress-test-sim2h";

  script-stress-sim2h = pkgs.writeShellScriptBin name-stress-sim2h
  ''
  hc-cli-install
  hc-conductor-install
   ( cd stress-test && npm install && APP_SPEC_NETWORK_TYPE=sim2h npm test )
  '';
in
{
 buildInputs = [ script-stress-sim1h script-stress-sim2h ];
}
