let
 holonix-release-tag = "2019-06-26-windows-npm-debug";
 holonix-release-sha256 = "0mhrp677p45ihajajanav7cjvfhb2qn4g262vr06wy1zkj20mm0g";

 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/tarball/${holonix-release-tag}?1";
  # sha256 = "${holonix-release-sha256}";
 });
 # holonix = import ../holonix;
in
with holonix.pkgs;
{
 core-shell = stdenv.mkDerivation (holonix.shell // {
  name = "core-shell";

  shellHook = ''

   # cargo should install binaries into this repo rather than globally

   # https://github.com/rust-lang/rustup.rs/issues/994

   export CARGO_HOME=~/.cargo

   export CARGO_INSTALL_ROOT=~/.cargo

   export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"

   export HC_TARGET_PREFIX=~/nix-holochain/

  '';

  buildInputs = []
   ++ holonix.shell.buildInputs
   ++ (holonix.pkgs.callPackage ./release {
    holonix = holonix;
    pkgs = holonix.pkgs;
   }).buildInputs
  ;
 });
}
