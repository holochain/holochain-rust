{ pkgs, release }:
let
 manifest-versions = pkgs.callPackage ./nix/manifest-versions.nix {
  release = release;
 };
in
{
 buildInputs = [
  manifest-versions
 ];
}
