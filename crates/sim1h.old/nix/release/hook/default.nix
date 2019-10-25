{ pkgs, config }:
{
 buildInputs = []

 ++ (pkgs.callPackage ./publish {
  config = config;
 }).buildInputs

 ++ (pkgs.callPackage ./version {
  config = config;
 }).buildInputs
 ;
}
