let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "cli";
 sha256 = if pkgs.stdenv.isDarwin
 then
  # TODO - this hash will be wrong on mac!
  "15frnjn3q4mfsg53dy59mwnkhzwkf6iwm0d5jix2d575i8cyn5xi"
 else
  "15frnjn3q4mfsg53dy59mwnkhzwkf6iwm0d5jix2d575i8cyn5xi"
 ;
 binary = "hc";
}
