let
  rust = import ./config.nix;
  pkgs = import ../nixpkgs/nixpkgs.nix;
  build = (pkgs.rustChannelOfTargets "nightly" rust.nightly-date [ rust.wasm-target rust.generic-linux-target  ]);
in
[ build ]
# https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md
++ [ pkgs.binutils pkgs.gcc pkgs.gnumake pkgs.openssl pkgs.pkgconfig ]
