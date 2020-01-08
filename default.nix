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

    shellHook = holonix.pkgs.lib.concatStrings [
    holonix.shell.shellHook
    ''
    # environment variables used by rust tests directly
    export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
    export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
    # config file used by aws cli tool
    export AWS_CONFIG_FILE=`pwd`/.aws/config
    RUST_LOG=sim1h=trace
    export HC_TARGET_PREFIX=$NIX_ENV_PREFIX
    export CARGO_TARGET_DIR="$HC_TARGET_PREFIX/target"
    export CARGO_CACHE_RUSTC_INFO=1
    ''
    ];

  buildInputs = [ holonix.pkgs.libiconv ]
   ++ holonix.shell.buildInputs

   ++ (holonix.pkgs.callPackage ./app_spec_proc_macro {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/holochain {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/holochain_wasm {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/cli {
    pkgs = holonix.pkgs;
    config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/trycp_server {
     pkgs = holonix.pkgs;
     config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/sim2h_server {
     pkgs = holonix.pkgs;
     config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./crates/metrics {
    pkgs = holonix.pkgs;
    config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./docker {
     pkgs = holonix.pkgs;
   }).buildInputs

   # release hooks
   ++ (holonix.pkgs.callPackage ./release {
    holonix = holonix;
    pkgs = holonix.pkgs;
    config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./rust {
    holonix = holonix;
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./stress-test {
    pkgs = holonix.pkgs;
   }).buildInputs

   # main test script
   ++ (holonix.pkgs.callPackage ./test {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./.aws {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./dynamodb {
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
