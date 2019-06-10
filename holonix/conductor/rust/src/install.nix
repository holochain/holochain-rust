let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-conductor-rust-install";

  script = pkgs.writeShellScriptBin name
  ''
  cargo build -p holochain --release && cargo install -f --path conductor
  '';
in
script
