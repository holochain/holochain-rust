# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
with import <nixpkgs> {};

let
  rustVersionTools = "nightly-2018-07-17";
  hc-fmt = pkgs.writeShellScriptBin "hc-fmt" "rustup run ${rustVersionTools} cargo fmt";
in
stdenv.mkDerivation rec {
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
    hc-fmt
  ];

}
