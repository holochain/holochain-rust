let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  name = "hc-flush";

  script = pkgs.writeShellScriptBin name
  ''
  hc-node-flush
  hc-rust-flush
  hc-qt-c-bindings-flush
  '';
in
script
