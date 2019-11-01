{ pkgs, config }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./install { }).buildInputs
 ++ (pkgs.callPackage ./release {
  pkgs = pkgs;
  config = config;
 }).buildInputs
 ++ (pkgs.callPackage ./test { }).buildInputs
 ++ (pkgs.callPackage ./uninstall { }).buildInputs
 ;
}
