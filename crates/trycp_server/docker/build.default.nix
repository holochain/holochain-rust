let
 config = { };
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.39.tar.gz;
  sha256 = "1c8jk9cmpayjy7bndymx5c7wshwgd7a2xqda4lvp2y7ds72i8jhz";
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
