let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;
  lib = import ../lib.nix;
  dist-rust = import ../config.nix;

  name = "hc-dist-rust-compile";

  artifact-list = [
   dist-rust.cli
   dist-rust.conductor
  ];

  script = pkgs.writeShellScriptBin name
  ''
   ${pkgs.lib.concatMapStrings (artifact: lib.build-rust-artifact artifact) artifact-list}
  '';
in
script
