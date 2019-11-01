let
 holonix-release-tag = "further-bad-assigning";
 holonix-release-sha256 = "1mhrp677p45ihajnjanav7cjvfhb2qn4g262vr06wy1zkj20mm0g";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/archive/${holonix-release-tag}.tar.gz";
  sha256 = "${holonix-release-sha256}";
 });
in
with holonix.pkgs;
{
 core-shell = stdenv.mkDerivation (holonix.shell // {
  name = "dev-shell";

  buildInputs = []
  ++ holonix.shell.buildInputs
  ;
 });
}
