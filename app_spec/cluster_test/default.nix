{ pkgs }:
let
  name-cluster = "hc-app-spec-cluster-test";

  script-cluster = pkgs.writeShellScriptBin name-cluster
  ''
  hc-cli-install
  hc-conductor-install
   ( cd hc_cluster_test && npm install && ./node_modules/.bin/tsc)
   ( cd app_spec && mkdir -p dist && hc package --output dist/app_spec.dna.json )
   ( EMULATION_HOLOCHAIN_BIN_PATH=$CARGO_INSTALL_ROOT/bin/holochain node ./app_spec/cluster_test/index.js 2)
  '';
in
{
 buildInputs = [ script-cluster ];
}
