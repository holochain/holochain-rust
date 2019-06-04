let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-conductor-rust-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping holochain binary from cargo home directory"
   rm -f $CARGO_HOME/bin/holochain
  '';
in
script
