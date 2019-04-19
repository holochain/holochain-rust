let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-fmt-check";

  script = pkgs.writeShellScriptBin name
  ''
   cargo fmt -- --check
  '';
in
script
