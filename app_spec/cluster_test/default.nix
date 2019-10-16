{ pkgs }:
let
  name-cluster = "hc-app-spec-cluster-test";

  script-cluster = pkgs.writeShellScriptBin name-cluster
  ''
  hc-cli-install
  hc-conductor-install
   ( cd hc_cluster_test && npm install && ./node_modules/.bin/tsc)
   ( cd app_spec && mkdir -p dist && hc package --output dist/app_spec.dna.json --strip-meta )
   ( EMULATION_HOLOCHAIN_BIN_PATH=./.cargo/bin/holochain node ./app_spec/cluster_test/index.js 2)
  '';

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
 buildInputs = [ script-cluster script-stress-sim1h script-stress-sim2h ];
}
