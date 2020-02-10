let
 config = import ../config.nix;
 holonix = import (fetchTarball {
  url = "https://github.com/holochain/holonix/archive/${config.holonix.github.ref}.tar.gz";
  sha256 = config.holonix.github.sha256;
  }) { };
 shell-config = {
  LIBCLANG_PATH="${holonix.pkgs.llvmPackages.libclang}/lib";

 # needed for newrelic to compile its dependencies
 # this is a hack to workaround this:
 # https://github.com/NixOS/nixpkgs/issues/18995
 hardeningDisable = [ "fortify" ];
  CARGO_HOME = "/holochain/.cargo";
  name = "dev-shell";
  buildInputs = [ holonix.pkgs.pcre holonix.pkgs.cmake holonix.pkgs.clang
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
