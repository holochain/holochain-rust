# This is an example of what downstream consumers of holonix should do
# This is also used to dogfood as many commands as possible for holonix
# For example the release process for holonix uses this file
let

 # point this to your local config.nix file for this project
 # example.config.nix shows and documents a lot of the options
 config = import ./config.nix;

 # START HOLONIX IMPORT BOILERPLATE
 holonix = import (
  if ! config.holonix.use-github
  then config.holonix.local.path
  else fetchTarball {
   url = "https://github.com/${config.holonix.github.owner}/${config.holonix.github.repo}/tarball/${config.holonix.github.ref}";
   sha256 = config.holonix.github.sha256;
  }
 ) { config = config; };
 # END HOLONIX IMPORT BOILERPLATE

in
with holonix.pkgs;
{
 dev-shell = stdenv.mkDerivation (holonix.shell // {
  name = "dev-shell";
  shellHook = holonix.pkgs.lib.concatStrings
    [
      holonix.shell.shellHook
      ''export HC_TARGET_PREFIX=~/.holochain-compile-cache/
      ''
    ];

  buildInputs = [ ]
   ++ holonix.shell.buildInputs

   ++ (holonix.pkgs.callPackage ./app_spec {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./app_spec_proc_macro {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./conductor {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./conductor_wasm {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./cli {
    pkgs = holonix.pkgs;
    config = config;
   }).buildInputs

   # qt specific testing
   ++ (holonix.pkgs.callPackage ./qt {
    pkgs = holonix.pkgs;
   }).buildInputs

   # release hooks
   ++ (holonix.pkgs.callPackage ./release {
    pkgs = holonix.pkgs;
    config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./rust {
    holonix = holonix;
    pkgs = holonix.pkgs;
   }).buildInputs

   # main test script
   ++ (holonix.pkgs.callPackage ./test {
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
