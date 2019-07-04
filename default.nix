let
 holonix-release-tag = "0.0.3";
 holonix-release-sha256 = "0da3kam3sxri73rfanlr8mkl95q74cqvn02y3fa0c021144qxgxv";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/tarball/${holonix-release-tag}";
  sha256 = "${holonix-release-sha256}";
 });
 # holonix = import ../holonix;
in
with holonix.pkgs;
{
 core-shell = stdenv.mkDerivation (holonix.shell // {
  name = "core-shell";

  buildInputs = []
   ++ holonix.shell.buildInputs
   ++ (holonix.pkgs.callPackage ./release {
    holonix = holonix;
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
