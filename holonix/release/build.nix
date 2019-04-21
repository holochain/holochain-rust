let
  prepare = import ./src/prepare.nix;
in
[
  prepare
]
++ import ./docs/build.nix
++ import ./github/build.nix
++ import ./npm/build.nix
++ import ./pulse/build.nix
