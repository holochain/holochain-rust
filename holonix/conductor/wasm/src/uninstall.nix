let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-conductor-wasm-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   cargo uninstall wasm-bindgen-cli
  '';
in
script
