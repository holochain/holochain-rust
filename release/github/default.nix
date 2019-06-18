{ pkgs, release }:
let
 github = import ./config.nix;
in
github // {
 buildInputs = []

 ++ (pkgs.callPackage ./merge {
  github = github;
  release = release;
 }).buildInputs

 ++ (pkgs.callPackage ./check-artifacts {
  release = release;
 }).buildInputs
 ;
}
