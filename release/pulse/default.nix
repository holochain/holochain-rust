{ pkgs, release, github }:
let
 pulse = import ./config.nix;
in
pulse // {
 buildInputs = []
 ++ (pkgs.callPackage ./tag {
  pulse = pulse;
  release = release;
  github = github;
 }).buildInputs
 ;
}
