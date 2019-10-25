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

 # make dynamodb derivation available to downstream users of sim1h
 dynamodb = (holonix.pkgs.callPackage ./dynamodb {
   pkgs = holonix.pkgs;
 })
 ;

in
with holonix.pkgs;
{
 dynamodb = dynamodb;

 dev-shell = stdenv.mkDerivation (holonix.shell // {
  name = "dev-shell";

  shellHook = holonix.pkgs.lib.concatStrings [''
  # environment variables used by rust tests directly
  export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
  export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
  # config file used by aws cli tool
  export AWS_CONFIG_FILE=`pwd`/.aws/config
  RUST_LOG=sim1h=trace
  ''
  holonix.shell.shellHook
  ];

  buildInputs = [
   holonix.pkgs.ngrok
  ]
   ++ holonix.shell.buildInputs
   ++ config.buildInputs

   # release hooks
   ++ (holonix.pkgs.callPackage ./nix/release {
     pkgs = holonix.pkgs;
     config = config;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./.aws {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./.circleci {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ dynamodb.buildInputs
  ;
 });
}
