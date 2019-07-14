{ pkgs }:
{
 # assumes all the rust deps in holonix itself
 buildInputs = []
 ++ (pkgs.callPackage ./wasm { }).buildInputs
 ;
}
