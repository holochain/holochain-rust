let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-manifest-install";

  script = pkgs.writeShellScriptBin name
  ''
   cargo install cargo-edit
  '';
in
script
