let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-app-spec-serve";

  script = pkgs.writeShellScriptBin name
  ''
   hc-conductor-rust-install
   holochain -c ./app_spec/container-config.toml
  '';
in
script
