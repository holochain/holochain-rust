{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./cluster_test { }).buildInputs
 ++ (pkgs.callPackage ./test { }).buildInputs
 ;
}
