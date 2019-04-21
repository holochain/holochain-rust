let
 lib = import ./lib.nix;
in
lib.binary-derivation {
 name = "conductor";
 sha256 = "0wdlv85vwwp9cwnmnsp20aafrxljsxlc6m00h0905q0cydsf86kq";
 binary = "holochain";
}
