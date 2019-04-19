let
  rust = import ./config.nix;
  pkgs = import ../nixpkgs/nixpkgs.nix;
  build = (pkgs.rustChannelOfTargets "nightly" rust.nightly-date [ rust.wasm-target rust.generic-linux-target  ]);
in
[ build ]
