let
  prepare = import ./src/prepare.nix;
in
[
  prepare
]
++ import ./github/build.nix
++ import ./npm/build.nix
++ import ./pulse/build.nix
