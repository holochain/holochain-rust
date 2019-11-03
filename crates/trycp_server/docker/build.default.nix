let
 holonix = import (fetchTarball {
  url = https://github.com/holochain/holonix/archive/v0.0.39.tar.gz;
  sha256 = "1c8jk9cmpayjy7bndymx5c7wshwgd7a2xqda4lvp2y7ds72i8jhz";
  }) { config = {}; };
in
{
 dev-shell = holonix.pkgs.stdenv.mkDerivation({
  buildInputs = [];
 });

 holochain = {
  hc = holonix.holochain.hc;
  holochain = holonix.holochain.holochain;
 };

}
