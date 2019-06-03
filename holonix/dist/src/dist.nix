let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-dist";

  script = pkgs.writeShellScriptBin name
  ''
  hc-cli-dist
  hc-conductor-node-dist
  hc-conductor-rust-dist
  '';
in
script
