let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-app-spec-test";

  script = pkgs.writeShellScriptBin name
  ''
   ( cd app_spec && . build_and_test.sh )
  '';
in
script
