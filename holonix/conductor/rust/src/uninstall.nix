let
  pkgs = import ../../../nixpkgs/nixpkgs.nix;

  name = "hc-conductor-rust-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping holochain binary from home directory"
   rm -f ~/.cargo/bin/holochain
  '';
in
script
