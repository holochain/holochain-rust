let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-cli-test";

  script = pkgs.writeShellScriptBin name
  ''
  (cd cli && cargo test);
  '';
in
script
