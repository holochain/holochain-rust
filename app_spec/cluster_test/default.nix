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

  name-stress = "hc-stress-test-sim1h";

  script-stress = pkgs.writeShellScriptBin name-stress
  ''
  hc-cli-install
  hc-conductor-install
   ( cd stress-test && npm install && AWS_ACCESS_KEY_ID=bla AWS_SECRET_ACCESS_KEY=blup npm test )
  '';
in
{
 buildInputs = [ script-cluster script-stress ];
}
