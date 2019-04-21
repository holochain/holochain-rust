let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "conductor";
 sha256 = if pkgs.stdenv.isDarwin
 then
  # TODO - this hash will be wrong on mac!
  "0wdlv85vwwp9cwnmnsp20aafrxljsxlc6m00h0905q0cydsf86kq"
 else
  "0wdlv85vwwp9cwnmnsp20aafrxljsxlc6m00h0905q0cydsf86kq"
 ;
 binary = "holochain";
}
