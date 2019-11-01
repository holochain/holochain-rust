{ pkgs, config }:
{
 buildInputs = []

 ++ (pkgs.callPackage ./check-artifacts {
  pkgs = pkgs;
  config = config;
 }).buildInputs
 ;
}
