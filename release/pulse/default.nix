{ pkgs, release, github }:
let
 config = import ./nix/config.nix;

 tag = pkgs.callPackage ./nix/tag.nix {
   pulse = config;
   release = release;
   github = github;
 };
in
{
 config = import ./nix/config.nix;

 buildInputs =
 [
  tag
 ];
}
