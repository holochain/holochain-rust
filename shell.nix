# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "holochain-rust-environment";

  buildInputs = [
  pkgs.zeromq3
  pkgs.cargo
  pkgs.rustup
  pkgs.rustfmt
  pkgs.rustc
  pkgs.gnumake
  pkgs.gcc
  pkgs.binutils-unwrapped
  ];

  # The '' quotes are 2 single quote characters
  # They are used for multi-line strings
  shellHook = ''
  '';
}
