{ pkgs, release }:
let
 changelog-versions = pkgs.callPackage ./nix/changelog-versions.nix {
  release = release;
 };
in
{
 buildInputs = [
  changelog-versions
 ];
}
