let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-fmt";

  script = pkgs.writeShellScriptBin name
  ''
   cargo fmt
  '';
in
script
