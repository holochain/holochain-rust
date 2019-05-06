let
 pkgs = import ../../../nixpkgs/nixpkgs.nix;
 lib = import ../../src/lib.nix;

 args = import ../config.nix;
in
lib.binary-derivation args
