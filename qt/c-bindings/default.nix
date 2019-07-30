{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./flush { }).buildInputs
 ++ (pkgs.callPackage ./test { }).buildInputs
 ;
}
