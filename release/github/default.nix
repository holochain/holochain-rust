{ pkgs, release }:
let
 config = import ./nix/config.nix;

 merge = pkgs.callPackage ./nix/merge.nix {
  github = config;
  release = release;
 };
in
{
 config = config;
 buildInputs = [
  merge
 ];
}
