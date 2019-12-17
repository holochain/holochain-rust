{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./cluster_test { }).buildInputs
 ;
}
