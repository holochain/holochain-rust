{ holonix, pkgs }:
{
 # assumes all the rust deps in holonix itself
 buildInputs = []
 ++ (pkgs.callPackage ./test { holonix = holonix; }).buildInputs
 ++ (pkgs.callPackage ./fmt { }).buildInputs
 ++ (pkgs.callPackage ./clippy { }).buildInputs
 ++ (pkgs.callPackage ./wasm { }).buildInputs
 ;
}
