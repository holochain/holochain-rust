let
 config = { };
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.52.tar.gz;
  sha256 = "02x16zyspx8as65z5glmqsana4gmrkvjm9q8ws1f6l1wip87h2ql";
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
