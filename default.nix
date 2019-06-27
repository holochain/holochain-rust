let
 holonix-release-tag = "2019-06-27-rust-nightly";
 holonix-release-sha256 = "1n26n9q4i2k11n1m7disjs7s5s11lq29icqyk8qngqs5gf7kq4pi";

 hxolonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/tarball/${holonix-release-tag}";
  # sha256 = "${holonix-release-sha256}";
 });
 holonix = import ../holonix;
in
with holonix.pkgs;
{
 core-shell = stdenv.mkDerivation (holonix.shell // {
  name = "core-shell";

  buildInputs = []
   ++ holonix.shell.buildInputs

   ++ (holonix.pkgs.callPackage ./qt {
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./release {
    holonix = holonix;
    pkgs = holonix.pkgs;
   }).buildInputs

   ++ (holonix.pkgs.callPackage ./test {
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
