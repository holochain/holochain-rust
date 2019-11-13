{ pkgs, config }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./version {
  pkgs = pkgs;
  config = config;
 }).buildInputs
 ;
}
