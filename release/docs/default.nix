{ pkgs, release }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./changelog-versions {
  release = release;
 }).buildInputs
 ;
}
