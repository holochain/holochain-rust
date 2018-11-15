# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
with import <nixpkgs> {};

let
  rustVersionCore = "nightly-2018-10-12";
  rustVersionTools = "nightly-2018-07-17";
  hc-fmt = pkgs.writeShellScriptBin "hc-fmt" "rustup run ${rustVersionTools} cargo fmt";
  hc-test = pkgs.writeShellScriptBin "hc-test" "rustup run ${rustVersionCore} cargo test";
in
stdenv.mkDerivation rec {
  name = "holochain-rust-environment";

  buildInputs = [
    pkgs.zeromq
    pkgs.rustup
    pkgs.cargo
    pkgs.rustfmt
    pkgs.rustc
    pkgs.gnumake
    pkgs.gcc
    pkgs.binutils-unwrapped
    pkgs.pkgconfig
    pkgs.python
    hc-fmt
    hc-test
  ];

}
