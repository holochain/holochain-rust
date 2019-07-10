let
 holonix-release-tag = "0.0.3-wasm-opt-2";
 holonix-release-sha256 = "01mq5z6fqbcy3xmdva20r4l0dqbzlq9w6dl9vxr4pkcw2av2vqas";

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
