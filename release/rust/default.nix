{ pkgs, release }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./manifest-versions {
  release = release;
 }).buildInputs
 ;
}
