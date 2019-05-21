let
  rust = import ./config.nix;
  pkgs = import ../nixpkgs/nixpkgs.nix;
  build = (pkgs.rustChannelOfTargets "nightly" rust.nightly.date [ rust.wasm-target rust.generic-linux-target  ]);

  flush = import ./src/flush.nix;
  fmt-check = import ./src/fmt-check.nix;
  fmt = import ./src/fmt.nix;
  test = import ./src/test.nix;
in
[ build ]
++ import ./coverage/build.nix
++ import ./fmt/build.nix
++ import ./manifest/build.nix
++ import ./wasm/build.nix
# https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md
++ [
  pkgs.binutils
  pkgs.gcc
  pkgs.gnumake
  pkgs.openssl
  pkgs.pkgconfig
]
++ [
  flush
  test
]
