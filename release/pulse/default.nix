{ holonix, pkgs, release, github, rust }:
let
 pulse = import ./config.nix;
in
pulse // {
 buildInputs = []

 ++ (pkgs.callPackage ./notes {
  holonix = holonix;
  release = release;
  pulse = pulse;
 }).buildInputs

 ++ (pkgs.callPackage ./sync {
  release = release;
  github = github;
 }).buildInputs

 ++ (pkgs.callPackage ./tag {
  pulse = pulse;
  release = release;
  github = github;
 }).buildInputs

 ;
}
