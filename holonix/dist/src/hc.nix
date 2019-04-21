let
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "cli";
 sha256 = "15frnjn3q4mfsg53dy59mwnkhzwkf6iwm0d5jix2d575i8cyn5xi";
 binary = "hc";
}
