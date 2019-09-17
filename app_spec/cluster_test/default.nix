{ pkgs }:
let
  name = "hc-app-spec-cluster-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-rust-install
   ( cd hc_cluster_test && npm install && ./node_modules/.bin/tsc)
   ( cd app_spec && mkdir -p dist && hc package --output dist/app_spec.dna.json --strip-meta )
   ( EMULATION_HOLOCHAIN_BIN_PATH=./.cargo/bin/holochain node ./app_spec/cluster_test/index.js 2)
  '';
in
{
 buildInputs = [ script ];
}
