{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./test { }).buildInputs
 ;
}
