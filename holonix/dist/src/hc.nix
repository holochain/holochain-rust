let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "cli";
 sha256 = if pkgs.stdenv.isDarwin
 then
  "0n0cq0b9x0jblbydjrpfql7qminkrnxnq8hcb6kb57q08i71pwza"
 else
  "15frnjn3q4mfsg53dy59mwnkhzwkf6iwm0d5jix2d575i8cyn5xi"
 ;
 binary = "hc";
}
