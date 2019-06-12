let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-cli-install";

  script = pkgs.writeShellScriptBin name
  ''
  cargo build -p hc --release && cargo install -f --path cli
  '';
in
script
