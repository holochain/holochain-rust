{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./compile { }).buildInputs
 ;
}
