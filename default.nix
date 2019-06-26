let
 holonix-release-tag = "2019-06-26-windows-npm-debug?5";
 holonix-release-sha256 = "0mhrp677p45ihajajanav7cjvfhb2qn4g262vr06wy1zkj20mm0g";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/tarball/${holonix-release-tag}";
  # sha256 = "${holonix-release-sha256}";
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
