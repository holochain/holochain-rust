let
 config = import ../config.nix;
 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/archive/${config.holonix.github.ref}.tar.gz";
  sha256 = config.holonix.github.sha256;
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
