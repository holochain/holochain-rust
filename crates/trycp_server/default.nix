{ pkgs, config }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./docker { }).buildInputs
 ++ (pkgs.callPackage ./install { }).buildInputs
 ++ (pkgs.callPackage ./uninstall { }).buildInputs
 ;
}
