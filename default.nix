let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;

  cli = import ./holonix/dist/cli/build.nix;
  conductor = import ./holonix/dist/conductor/build.nix;

in
with pkgs;
stdenv.mkDerivation rec {

 name = "holochain-binaries";

 buildInputs = [
  cli
  conductor
 ];

}
