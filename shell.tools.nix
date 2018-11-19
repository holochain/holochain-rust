# This imports the nix package collection,
# so we can access the `pkgs` and `stdenv` variables
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  date = "2018-07-17";

  rust-build = (nixpkgs.rustChannelOf {channel = "nightly"; date = date;}).rust;

  hc-fmt = nixpkgs.writeShellScriptBin "hc-fmt" "cargo fmt";
  hc-fmt-check = nixpkgs.writeShellScriptBin "hc-fmt-check" "cargo fmt -- --check";

in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "holochain-tools-environment";

  buildInputs = [
    zeromq
    rust-build

    hc-fmt
    hc-fmt-check

    zeromq
  ];

  # https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md
  RUSTFLAGS = "-D warnings";

}
