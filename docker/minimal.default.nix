let
 config = { };
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.47.tar.gz;
  sha256 = "0fyal8y02abp71y0l3szcbc9d0r89ph5yk3d6maw28as78mw7v23";
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
