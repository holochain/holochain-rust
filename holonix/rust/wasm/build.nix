let
  pkgs = import ../../nixpkgs/nixpkgs.nix;
  compile = import ./src/compile.nix;
in
[
  # wabt needs cmake
  pkgs.cmake
  compile
]
