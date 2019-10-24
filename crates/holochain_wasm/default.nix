{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./compile { }).buildInputs
 ++ (pkgs.callPackage ./install { }).buildInputs
 ++ (pkgs.callPackage ./test { }).buildInputs
 ++ (pkgs.callPackage ./uninstall { }).buildInputs
 ;
}
