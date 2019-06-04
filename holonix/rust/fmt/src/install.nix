let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-rust-fmt-install";

  # TODO - currently broken! nix doesn't play nice with rustup :'(
  script = pkgs.writeShellScriptBin name
  ''
  rustup component add rustfmt
  '';
in
script
