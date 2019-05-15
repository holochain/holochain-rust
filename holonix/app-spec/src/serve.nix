let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-app-spec-serve";

  script = pkgs.writeShellScriptBin name
  ''
   hc-conductor-rust-install
   holochain -c ./app_spec/conductor/''$1.toml
  '';
in
script
