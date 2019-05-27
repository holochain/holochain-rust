let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-app-spec-cluster-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-rust-install
   ( cd hc_cluster_test && npm install )
   ( cd app_spec && hc package --output dist/app_spec.dna.json --strip-meta )
   ( EMULATION_HOLOCHAIN_BIN_PATH=./.cargo/bin/holochain node ./app_spec/cluster_test/index.js )
  '';
in
script
