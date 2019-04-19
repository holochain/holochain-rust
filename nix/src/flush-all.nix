let
  pkgs = import ../nixpkgs/nixpkgs.nix;

  name = "hc-flush-all";

  script = pkgs.writeShellScriptBin name
  ''
   hc-node-flush
   hc-rust-flush
  '';
in
script
