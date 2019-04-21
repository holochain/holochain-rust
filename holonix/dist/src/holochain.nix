let
 pkgs = import ../../nixpkgs/nixpkgs.nix;
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "conductor";
 sha256 = if pkgs.stdenv.isDarwin
 then
  "012kga02mnci4vj92jxm1jp5w5z8x0phh4s7hbg0vihk56png43n"
 else
  "0wdlv85vwwp9cwnmnsp20aafrxljsxlc6m00h0905q0cydsf86kq"
 ;
 binary = "holochain";
}
