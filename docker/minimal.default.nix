let
 config = { };
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.47.tar.gz;
  sha256 = "0fyal8y02abp71y0l3szcbc9d0r89ph5yk3d6maw28as78mw7v23";
  }) { };
 shell-config = {
  CARGO_HOME = "/holochain/.cargo";
  name = "dev-shell";
  buildInputs = [
  ]
  ++ holonix.rust.buildInputs
  ;
 };
in
{
 shell-config = shell-config;
 dev-shell = holonix.pkgs.stdenv.mkDerivation shell-config;

 holochain = {
  holochain = holonix.holochain.holochain;
 };

 holonix = holonix;

}
