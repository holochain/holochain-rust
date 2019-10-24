{ pkgs, config }:
{
 buildInputs = []
 ++ (pkgs.callPackage ./hook {
  pkgs = pkgs;
  config = config;
 }).buildInputs
 ;
}
