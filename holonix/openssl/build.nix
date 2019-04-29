let
  pkgs = import ../nixpkgs/nixpkgs.nix;
in
[
  # the OpenSSL static installation provided by native-tls rust module on linux
  # environments uses perl under the hood to configure and install the
  # statically linked openssl lib
  pkgs.perl
]
