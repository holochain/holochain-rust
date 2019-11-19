let
 config = { };
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.44.tar.gz;
  sha256 = "0819439idwhdbavmlcy99c2ai5d9a0k7rbimbsk47p9vndw3s6cy";
  }) { };
in
{
 dev-shell = holonix.pkgs.stdenv.mkDerivation({
  CARGO_HOME = "/holochain";
  name = "dev-shell";
  buildInputs = [
   holonix.pkgs.wget
  ]
  ++ holonix.rust.buildInputs
  ++ (holonix.pkgs.callPackage ../default.nix {
   config = config;
   pkgs = holonix.pkgs;
  }).buildInputs
  ;
 });

 holochain = {
  hc = holonix.holochain.hc;
  holochain = holonix.holochain.holochain;
 };

}
