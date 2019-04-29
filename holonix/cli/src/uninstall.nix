let
  pkgs = import ../../nixpkgs/nixpkgs.nix;

  name = "hc-cli-uninstall";

  script = pkgs.writeShellScriptBin name
  ''
   echo "dropping hc binary from home directory"
   rm -f ~/.cargo/bin/hc
  '';
in
script
