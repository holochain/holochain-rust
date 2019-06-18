let
 holonix-release-tag = "predicted-betting-match";
 holonix-release-sha256 = "19pigw9ys14hfx5n5icbnr7bwgvjirr6x7yd3p46jph43h0rx9ih";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/archive/${holonix-release-tag}.tar.gz";
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
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
