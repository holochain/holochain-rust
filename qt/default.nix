{ pkgs }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./c-bindings { }).buildInputs
 ;
}
