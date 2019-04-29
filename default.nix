let

  pkgs = import ./holonix/nixpkgs/nixpkgs.nix;

in
with pkgs;
stdenv.mkDerivation rec {

 name = "holonix-binaries";

 buildInputs = import ./holonix/dist/build.nix;

}
