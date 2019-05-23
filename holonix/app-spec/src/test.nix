let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-app-spec-test";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-install
  hc-conductor-node-install
  (cd app_spec && . build_and_test.sh)
  '';
in
script
